use std::collections::{VecDeque, HashMap};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher, DefaultHasher};

use async_trait::async_trait;
use clap::{Args, ValueEnum};
use itertools::Itertools;

use super::{
    interface::{
        BasicMarkup, PipelineCommand, PipelineValues, SymbolTreeTable, SymbolTreeTableCell, SymbolTreeTableList, SymbolTreeTableNode,
        SymbolCrossrefInfo, SymbolTreeTableColumn,
    },
    symbol_graph::{
        DerivedSymbolInfo, SymbolGraphNodeId,
    },
};

use crate::file_format::analysis::{
    StructuredFieldInfo, StructuredBitPositionInfo,
};

use crate::abstract_server::{AbstractServer, ErrorDetails, ErrorLayer, Result, ServerError};

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum SymbolFormatMode {
    FieldLayout,
    // - class-field-use-matrix: table for each class, look up all its methods and all its
    //   fields, then filter the method "calls" to the fields.
    // - caller-matrix: look up a class, get all its methods.  look up all of
    //   the callers of all of those methods.  group them by their class.
    //   - row depth 0 is subsystem
    //   - row depth 1 is class or file if no class
    //   - row depth 2 is method/function
    //   - columns are the methods on the class, probably alphabetical.
    //     - columns could maybe have an upsell to the arg-matrix?
    //   - cells are a count.
    // - arg-matrix:
    //   - like caller-matrix but only for a single matrix and the columns are
    //     the args.
}

/// Given a list of symbol crossref infos, produce a SymbolTreeTable for display
/// purposes.
#[derive(Debug, Args)]
pub struct FormatSymbols {
    #[clap(long, value_parser, value_enum, default_value = "field-layout")]
    pub mode: SymbolFormatMode,
}

#[derive(Debug)]
pub struct FormatSymbolsCommand {
    pub args: FormatSymbols,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct PlatformId(u32);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct PlatformGroupId(u32);

// A struct to represent single field and hole before the field.
#[derive(Clone, Eq, Hash, PartialEq)]
struct Field {
    class_id: SymbolGraphNodeId,
    field_id: SymbolGraphNodeId,
    type_pretty: String,
    pretty: String,
    lineno: u64,
    hole_bytes: Option<u32>,
    hole_after_base: bool,
    end_padding_bytes: Option<u32>,
    offset_bytes: u32,
    bit_positions: Option<StructuredBitPositionInfo>,
    size_bytes: Option<u32>,
}

impl Field {
    fn new(class_id: SymbolGraphNodeId, field_id: SymbolGraphNodeId,
           sym_info: &DerivedSymbolInfo, info: &StructuredFieldInfo) -> Self {
        Self {
            class_id: class_id,
            field_id: field_id,
            type_pretty: info.type_pretty.to_string(),
            pretty: info.pretty.to_string(),
            lineno: sym_info.get_def_lno(),
            hole_bytes: None,
            hole_after_base: false,
            end_padding_bytes: None,
            offset_bytes: info.offset_bytes,
            bit_positions: info.bit_positions.clone(),
            size_bytes: info.size_bytes.clone(),
        }
    }
}

struct ClassSize {
    main_size: u32,
    per_platform: HashMap<PlatformId, u32>,
}

impl ClassSize {
    fn new(size: u32) -> Self {
        Self {
            main_size: size,
            per_platform: HashMap::new(),
        }
    }

    fn set_per_platform(&mut self, platform_id: PlatformId, size: u32) {
        self.per_platform.insert(platform_id, size);
    }
}

struct ClassSizeMap {
    map: HashMap<SymbolGraphNodeId, ClassSize>,
}

impl ClassSizeMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn set(&mut self, class_id: SymbolGraphNodeId, size: u32) {
        self.map.insert(class_id, ClassSize::new(size));
    }

    fn set_per_platform(&mut self, platform_id: PlatformId, class_id: SymbolGraphNodeId, size: u32) {
        if let Some(class_size) = self.map.get_mut(&class_id) {
            class_size.set_per_platform(platform_id, size);
            return;
        }

        let mut class_size = ClassSize::new(size.clone());
        class_size.set_per_platform(platform_id, size);
        self.map.insert(class_id, class_size);
    }

    fn main(&self) -> HashMap<SymbolGraphNodeId, u32> {
        let mut result = HashMap::new();

        for (class_id, class_size) in &self.map {
            result.insert(class_id.clone(), class_size.main_size);
        }

        result
    }

    fn per_platform(&self, platform_id: &PlatformId) -> HashMap<SymbolGraphNodeId, u32> {
        let mut result = HashMap::new();

        for (class_id, class_size) in &self.map {
            let size = match class_size.per_platform.get(platform_id) {
                Some(size) => *size,
                None => class_size.main_size,
            };
            result.insert(class_id.clone(), size);
        }

        result
    }
}

// A container for fields, with pre-calculated hash of fields.
struct FieldsWithHash {
    fields: Vec<Field>,
    hash: u64,
}

impl FieldsWithHash {
    fn new() -> Self {
        Self {
            fields: vec![],
            hash: 0,
        }
    }

    fn new_with_field(field: Field) -> Self {
        Self {
            fields: vec![field],
            hash: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    fn calculate_holes(&mut self, class_size_map: HashMap<SymbolGraphNodeId, u32>) {
        self.fields.sort_by(|a, b| {
            let byte_result = a.offset_bytes.cmp(&b.offset_bytes);
            if byte_result != Ordering::Equal {
                return byte_result;
            }

            match (&a.bit_positions, &b.bit_positions) {
                (Some(a_pos), Some(b_pos)) => {
                    a_pos.begin.cmp(&b_pos.begin)
                }
                _ => byte_result
            }
        });

        let mut last_end_offset = 0;
        let mut last_index = 0;

        let len = self.fields.len();

        for index in 0..len {
            if self.fields[index].offset_bytes > last_end_offset {
                if index != last_index {
                    if self.fields[last_index].class_id != self.fields[index].class_id {
                        let last_class_id = &self.fields[last_index].class_id;
                        if let Some(size) = class_size_map.get(last_class_id) {
                            if last_end_offset < *size {
                                self.fields[last_index].end_padding_bytes = Some(size - last_end_offset);
                            }
                            last_end_offset = *size;
                        }

                        self.fields[index].hole_after_base = true;
                    }
                }

                if self.fields[index].offset_bytes > last_end_offset {
                    self.fields[index].hole_bytes = Some(self.fields[index].offset_bytes - last_end_offset);
                }
            }

            last_index = index;

            if let Some(pos) = &self.fields[index].bit_positions {
                let end = self.fields[index].offset_bytes + (pos.begin + pos.width + 7) / 8;
                if end > last_end_offset {
                    last_end_offset = end;
                }
                continue;
            }

            if let Some(size) = &self.fields[index].size_bytes {
                last_end_offset = self.fields[index].offset_bytes + size;
            }
        }

        if !self.fields.is_empty() {
            let last_class_id = &self.fields[last_index].class_id;
            if let Some(size) = class_size_map.get(last_class_id) {
                if last_end_offset < *size {
                    self.fields[last_index].end_padding_bytes = Some(size - last_end_offset);
                }
            }
        }
    }

    fn calculate_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.fields.hash(&mut hasher);
        self.hash = hasher.finish();
    }
}

// A struct to represent single class, with
// fields per each platform group.
struct Class {
    id: SymbolGraphNodeId,
    name: String,
    supers: Vec<SymbolGraphNodeId>,
    fields: HashMap<SymbolGraphNodeId, HashMap<PlatformGroupId, Field>>,
    merged_fields: Vec<Vec<Option<Field>>>,
}

impl Class {
    fn new(id: SymbolGraphNodeId, name: String) -> Self {
        Self {
            id: id,
            name: name,
            supers: vec![],
            fields: HashMap::new(),
            merged_fields: vec![],
        }
    }

    fn add_field(&mut self, group_id: PlatformGroupId, field: Field) {
        let field_id = field.field_id.clone();

        if let Some(field_variants_map) = self.fields.get_mut(&field_id) {
            field_variants_map.insert(group_id, field);
            return;
        }

        let mut field_variants_map = HashMap::new();
        field_variants_map.insert(group_id, field);
        self.fields.insert(field_id, field_variants_map);
    }

    fn finish_populating(&mut self, groups: &Vec<(PlatformGroupId, Vec<PlatformId>)>) {
        // Sort the fields based on:
        //   * Line number
        //   * Average bit offset of the field
        //   * Integer encoding of the groups where the field exists

        let mut field_list = vec![];

        for field_variants_map in self.fields.values() {
            let mut group_bits: u64 = 0;
            let mut total_lineno: u64 = 0;
            let mut total_bit_offset: u64 = 0;
            let mut field_count: u64 = 0;

            let mut field_variants = vec![];
            for (group_id, _) in groups {
                match field_variants_map.get(group_id) {
                    Some(field) => {
                        total_lineno += field.lineno;
                        total_bit_offset += (field.offset_bytes as u64) * 8;
                        if let Some(pos) = &field.bit_positions {
                            total_bit_offset += pos.begin as u64;
                        }
                        group_bits |= 1 << group_id.0;

                        field_count += 1;

                        field_variants.push(Some(field.clone()));
                    },
                    None => {
                        field_variants.push(None);
                    },
                }
            }

            let average_lineno = total_lineno / field_count;
            let average_bit_offset = total_bit_offset / field_count;

            field_list.push((average_lineno, average_bit_offset, group_bits, field_variants))
        }

        field_list.sort_by(|a, b| {
            let result = a.0.cmp(&b.0);
            if result != Ordering::Equal {
                return result;
            }

            let result = a.1.cmp(&b.1);
            if result != Ordering::Equal {
                return result;
            }

            let result = a.2.cmp(&b.2);
            if result != Ordering::Equal {
                return result;
            }

            Ordering::Equal
        });

        self.merged_fields = field_list
            .into_iter()
            .map(|(_, _, _, field_variants)| field_variants)
            .collect();
    }
}

// Collect all platforms appeared in the analysis.
struct PlatformMap {
    platform_id_to_name: Vec<String>,

    // The temporary data structure to calculate platform ID.
    platform_name_to_id: HashMap<String, PlatformId>,
}

impl PlatformMap {
    fn new() -> Self {
        Self {
            platform_id_to_name: vec![],
            platform_name_to_id: HashMap::new(),
        }
    }

    fn add(&mut self, platform: String) -> PlatformId {
        if let Some(platform_id) = self.platform_name_to_id.get(&platform) {
            return platform_id.clone();
        }

        let platform_id = PlatformId(self.platform_name_to_id.len() as u32);
        self.platform_id_to_name.push(platform.clone());
        self.platform_name_to_id.insert(platform, platform_id.clone());

        platform_id
    }

    fn platform_ids(&self) -> Vec<PlatformId> {
        self.platform_id_to_name
            .iter()
            .enumerate()
            .map(|(i, _)| PlatformId(i as u32))
            .collect()
    }


    fn get_name(&self, platform_id: &PlatformId) -> String {
        self.platform_id_to_name[platform_id.0 as usize].clone()
    }
}

fn platform_name_to_order(name: &String) -> u32 {
    if name.starts_with("win") {
        return 0;
    }
    if name.starts_with("macosx") {
        return 1;
    }
    if name.starts_with("linux") {
        return 2;
    }
    if name.starts_with("android") {
        return 3;
    }
    if name.starts_with("ios") {
        return 4;
    }
    return 5;
}

// Struct to hold the list of fields for the entire class hierarchy
// per platform, and calculate the hole between them.
struct FieldsPerPlatform {
    platform_agnostic_fields: FieldsWithHash,
    fields_per_platform: HashMap<PlatformId, FieldsWithHash>,
}

impl FieldsPerPlatform {
    fn new() -> Self {
        Self {
            platform_agnostic_fields: FieldsWithHash::new(),
            fields_per_platform: HashMap::new(),
        }
    }

    fn add_field(&mut self, field: Field) {
        self.platform_agnostic_fields.fields.push(field);
    }

    fn add_field_per_platform(&mut self, platform_id: &PlatformId, field: Field) {
        if let Some(fields) = self.fields_per_platform.get_mut(platform_id) {
            fields.fields.push(field);
            return;
        }

        self.fields_per_platform.insert(platform_id.clone(), FieldsWithHash::new_with_field(field));
    }

    // Once all fields are populated, process them for further operation.
    fn finish_populating(&mut self, platform_map: &PlatformMap,
                         class_size_map: &ClassSizeMap) {
        if !self.platform_agnostic_fields.is_empty() && !self.fields_per_platform.is_empty() {
            // If there are per-platform field and also platform-agnostic field,
            // copy platform-agnostic fields into per-platform fields and perform
            // the remaining steps for per-platform fields.

            let platform_agnostic_fields: Vec<Field> = self.platform_agnostic_fields.fields.drain(..).collect();

            for platform_id in &platform_map.platform_ids() {
                for field in &platform_agnostic_fields {
                    self.add_field_per_platform(&platform_id, field.clone());
                }
            }
        }

        if self.fields_per_platform.is_empty() {
            self.platform_agnostic_fields.calculate_holes(class_size_map.main());
        } else {
            for (platform_id, fields) in self.fields_per_platform.iter_mut() {
                fields.calculate_holes(class_size_map.per_platform(&platform_id));
                fields.calculate_hash();
            }
        }
    }

    fn group_platforms(&self, platform_map: &PlatformMap) -> Vec<(PlatformGroupId, Vec<PlatformId>)> {
        if self.fields_per_platform.is_empty() {
            // If all fields are platform-agnostic, simply return them.
            return vec![(PlatformGroupId(0), vec![])];
        }

        // Group platforms by fields.
        let mut groups: Vec<(u64, Vec<PlatformId>)> = vec![];

        let mut platform_ids = platform_map.platform_ids();

        // Make the order consistent as much as possible across classes.
        platform_ids.sort_by(|a, b| {
            let a_name = platform_map.get_name(&a);
            let b_name = platform_map.get_name(&b);

            let a_order = platform_name_to_order(&a_name);
            let b_order = platform_name_to_order(&b_name);

            let result = a_order.cmp(&b_order);
            if result != Ordering::Equal {
                return result
            }

            a_name.cmp(&b_name)
        });

        'next_platform: for platform_id in &platform_ids {
            if let Some(fields) = self.fields_per_platform.get(&platform_id) {
                for (hash, platforms) in &mut groups {
                    if fields.hash == *hash {
                        let existing = &self.fields_per_platform.get(&platforms[0]).unwrap().fields;
                        if fields.fields == *existing {
                            platforms.push(platform_id.clone());
                            continue 'next_platform;
                        }
                    }
                }

                groups.push((fields.hash, vec![platform_id.clone()]));
            }
        }

        groups
            .into_iter()
            .enumerate()
            .map(|(i, (_, platforms))| (PlatformGroupId(i as u32), platforms))
            .collect()
    }

    fn get_fields_for_platforms<'a>(&'a self, platform_ids: &Vec<PlatformId>) -> &'a Vec<Field> {
        if platform_ids.is_empty() {
            return &self.platform_agnostic_fields.fields;
        }

        let platform_id = &platform_ids[0];
        &self.fields_per_platform.get(&platform_id).unwrap().fields
    }
}

struct ClassMap {
    // All processed classes.
    class_map: HashMap<SymbolGraphNodeId, Class>,

    // The list of classes, in the traverse order.
    class_list: Vec<SymbolGraphNodeId>,

    // All platforms appeared inside the analysis.
    platform_map: PlatformMap,

    // Platforms grouped by the field layout.
    groups: Vec<(PlatformGroupId, Vec<PlatformId>)>,

    root_sym_id: Option<SymbolGraphNodeId>,
    stt: SymbolTreeTable,
}

impl ClassMap {
    fn new() -> Self {
        Self {
            class_map: HashMap::new(),
            class_list: vec![],
            platform_map: PlatformMap::new(),
            groups: vec![],
            root_sym_id: None,
            stt: SymbolTreeTable::new(),
        }
    }

    async fn populate(&mut self, nom_sym_info: SymbolCrossrefInfo,
                      server: &Box<dyn AbstractServer + Send + Sync>) -> Result<()> {
        let (root_sym_id, _) = self.stt.node_set.add_symbol(DerivedSymbolInfo::new(
            nom_sym_info.symbol,
            nom_sym_info.crossref_info,
            0,
        ));

        self.root_sym_id = Some(root_sym_id.clone());

        let mut class_size_map = ClassSizeMap::new();

        let mut fields_per_platform = FieldsPerPlatform::new();

        let mut pending_ids = VecDeque::new();
        pending_ids.push_back(root_sym_id);

        while let Some(sym_id) = pending_ids.pop_front() {
            let sym_info = self.stt.node_set.get(&sym_id);
            let depth = sym_info.depth;
            let Some(structured) = sym_info.get_structured() else {
                continue;
            };

            let mut cls = Class::new(
                sym_id.clone(),
                structured.pretty.to_string(),
            );

            if let Some(size) = &structured.size_bytes {
                let platforms = structured.platforms();
                if platforms.is_empty() {
                    class_size_map.set(cls.id.clone(), *size);
                } else {
                    for platform in platforms {
                        let platform_id = self.platform_map.add(platform.clone());
                        class_size_map.set_per_platform(platform_id, cls.id.clone(), *size);
                    }
                }
            }

            let variants = structured.variants();
            for v in variants {
                if let Some(size) = &v.size_bytes {
                    let platforms = v.platforms();
                    for platform in platforms {
                        let platform_id = self.platform_map.add(platform.clone());
                        class_size_map.set_per_platform(platform_id, cls.id.clone(), *size);
                    }
                }
            }

            for super_info in &structured.supers {
                let (super_id, _) = self.stt
                    .node_set
                    .ensure_symbol(&super_info.sym, server, depth + 1)
                    .await?;
                cls.supers.push(super_id.clone());

                pending_ids.push_back(super_id);
            }
            self.class_list.push(cls.id.clone());
            self.class_map.insert(cls.id.clone(), cls);

            let platforms_and_fields = structured.fields_across_all_variants();
            for (platforms, fields) in platforms_and_fields {
                let mut platform_ids = vec![];

                for platform in &platforms {
                    let platform_id = self.platform_map.add(platform.clone());
                    platform_ids.push(platform_id);
                }

                for field in fields {
                    let (field_id, field_info) = self.stt
                        .node_set
                        .ensure_symbol(&field.sym, server, depth + 1)
                        .await?;

                    let field = Field::new(sym_id.clone(), field_id.clone(), field_info, &field);

                    if platform_ids.is_empty() {
                        fields_per_platform.add_field(field);
                    } else {
                        for platform_id in &platform_ids {
                            fields_per_platform.add_field_per_platform(&platform_id, field.clone());
                        }
                    }
                }
            }
        }

        fields_per_platform.finish_populating(&self.platform_map,
                                              &class_size_map);

        self.groups = fields_per_platform.group_platforms(&self.platform_map);

        for (group_id, platforms) in &self.groups {
            for field in fields_per_platform.get_fields_for_platforms(platforms) {
                let cls = self.class_map.get_mut(&field.class_id).unwrap();
                cls.add_field(group_id.clone(), field.clone());
            }
        }

        for cls in self.class_map.values_mut() {
            cls.finish_populating(&self.groups);
        }

        Ok(())
    }

    fn generate_tables(mut self, tables: &mut Vec<SymbolTreeTable>) {
        self.stt.columns.push(SymbolTreeTableColumn {
            label: vec![BasicMarkup::Heading("Name".to_string())],
            colspan: 1,
        });
        self.stt.columns.push(SymbolTreeTableColumn {
            label: vec![BasicMarkup::Heading("Type".to_string())],
            colspan: 1,
        });

        self.stt.sub_columns.push(SymbolTreeTableColumn {
            label: vec![],
            colspan: 1,
        });
        self.stt.sub_columns.push(SymbolTreeTableColumn {
            label: vec![],
            colspan: 1,
        });

        for (_, platforms) in &self.groups {
            let label = if platforms.is_empty() {
                "All platforms".to_string()
            } else {
                platforms
                    .iter()
                    .map(|platform_id| self.platform_map.get_name(&platform_id))
                    .join(" ")
                    .to_owned()
            };

            self.stt.columns.push(SymbolTreeTableColumn {
                label: vec![BasicMarkup::Heading(label)],
                colspan: 2,
            });

            self.stt.sub_columns.push(SymbolTreeTableColumn {
                label: vec![BasicMarkup::Text("Offset".to_string())],
                colspan: 1,
            });
            self.stt.sub_columns.push(SymbolTreeTableColumn {
                label: vec![BasicMarkup::Text("Size".to_string())],
                colspan: 1,
            });
        }

        let column_offset: usize = 1;

        let mut root_node: Option<SymbolTreeTableNode> = None;

        for class_id in &self.class_list {
            let cls = self.class_map.get(&class_id).unwrap();

            let mut class_node = SymbolTreeTableNode {
                sym_id: Some(cls.id.clone()),
                label: vec![BasicMarkup::Heading(cls.name.clone())],
                col_vals: vec![],
                children: vec![],
                colspan: (1 + column_offset + self.groups.len() * 2) as u32,
            };

            let field_prefix = format!("{}::", cls.name);

            for field_variants in &cls.merged_fields {
                let mut has_hole = false;
                for maybe_field in field_variants {
                    if let Some(field) = &maybe_field {
                        if field.hole_bytes.is_some() {
                            has_hole = true;
                            break;
                        }
                    }
                }

                if has_hole {
                    let mut hole_node = SymbolTreeTableNode {
                        sym_id: None,
                        label: vec![],
                        col_vals: vec![],
                        children: vec![],
                        colspan: 1,
                    };

                    hole_node.col_vals.push(SymbolTreeTableCell::empty());

                    for maybe_field in field_variants {
                        match maybe_field {
                            Some(field) => {
                                let hole_bytes = field.hole_bytes.unwrap_or(0);
                                if hole_bytes == 0 {
                                    hole_node.col_vals.push(SymbolTreeTableCell::empty_colspan(2));
                                    continue;
                                }

                                hole_node.col_vals.push(SymbolTreeTableCell::text_colspan(format!(
                                    "{} byte{} hole{}",
                                    hole_bytes,
                                    if hole_bytes == 1 {
                                        ""
                                    } else {
                                        "s"
                                    },
                                    if field.hole_after_base {
                                        " after super class"
                                    } else {
                                        ""
                                    }
                                ), 2));
                            },
                            None => {
                                if maybe_field.is_none() {
                                    hole_node.col_vals.push(SymbolTreeTableCell::empty_colspan(2));
                                }
                            }
                        }
                    }

                    class_node.children.push(hole_node);
                }

                let mut field_node = SymbolTreeTableNode {
                    sym_id: None,
                    label: vec![],
                    col_vals: vec![],
                    children: vec![],
                    colspan: 1,
                };

                field_node.col_vals.push(SymbolTreeTableCell::empty());

                for maybe_field in field_variants {
                    match maybe_field {
                        Some(field) => {
                            if field_node.sym_id.is_none() {
                                field_node.sym_id = Some(field.field_id.clone());

                                let mut pretty = field.pretty.clone();
                                pretty = pretty.replace(&field_prefix, "");
                                field_node.label = vec![BasicMarkup::Text(format!("{}", pretty))];

                                let type_label = match &field.type_pretty.is_empty() {
                                    false => format!("{}", field.type_pretty),
                                    true => "".to_string(),
                                };

                                field_node.col_vals[0].contents.push(BasicMarkup::Text(type_label));
                            }

                            field_node.col_vals.push(SymbolTreeTableCell::text(format!(
                                "{}",
                                field.offset_bytes,
                            )));

                            if let Some(pos) = &field.bit_positions {
                                field_node.col_vals.push(SymbolTreeTableCell::text(format!(
                                    "bits {}+{}",
                                    pos.begin, pos.width,
                                )));
                            } else {
                                field_node.col_vals.push(SymbolTreeTableCell::text(format!(
                                    "{}",
                                    field.size_bytes.unwrap_or(0),
                                )));
                            }
                        }
                        None => {
                            field_node.col_vals.push(SymbolTreeTableCell::empty());
                            field_node.col_vals.push(SymbolTreeTableCell::empty());
                        }
                    }
                }

                class_node.children.push(field_node);

                let mut has_end_padding = false;
                for maybe_field in field_variants {
                    if let Some(field) = &maybe_field {
                        if field.end_padding_bytes.is_some() {
                            has_end_padding = true;
                            break;
                        }
                    }
                }

                if has_end_padding {
                    let mut end_padding_node = SymbolTreeTableNode {
                        sym_id: None,
                        label: vec![],
                        col_vals: vec![],
                        children: vec![],
                        colspan: 1,
                    };

                    end_padding_node.col_vals.push(SymbolTreeTableCell::empty());

                    for maybe_field in field_variants {
                        match maybe_field {
                            Some(field) => {
                                let end_padding_bytes = field.end_padding_bytes.unwrap_or(0);
                                if end_padding_bytes == 0 {
                                    end_padding_node.col_vals.push(SymbolTreeTableCell::empty_colspan(2));
                                    continue;
                                }

                                end_padding_node.col_vals.push(SymbolTreeTableCell::text_colspan(format!(
                                    "{} byte{} padding",
                                    end_padding_bytes,
                                    if end_padding_bytes == 1 {
                                        ""
                                    } else {
                                        "s"
                                    }
                                ), 2));
                            },
                            None => {
                                if maybe_field.is_none() {
                                    end_padding_node.col_vals.push(SymbolTreeTableCell::empty_colspan(2));
                                }
                            }
                        }
                    }

                    class_node.children.push(end_padding_node);
                }
            }

            match &mut root_node {
                Some(node) => {
                    node.children.push(class_node);
                },
                None => {
                    root_node = Some(class_node);
                }
            }
        }

        if let Some(node) = root_node {
            self.stt.rows.push(node);
        }
        tables.push(self.stt);
    }
}

#[async_trait]
impl PipelineCommand for FormatSymbolsCommand {
    async fn execute(
        &self,
        server: &Box<dyn AbstractServer + Send + Sync>,
        input: PipelineValues,
    ) -> Result<PipelineValues> {
        let cil = match input {
            PipelineValues::SymbolCrossrefInfoList(cil) => cil,
            _ => {
                return Err(ServerError::StickyProblem(ErrorDetails {
                    layer: ErrorLayer::ConfigLayer,
                    message: "format-symbols needs a CrossrefInfoList".to_string(),
                }));
            }
        };

        match self.args.mode {
            SymbolFormatMode::FieldLayout => {
                let mut tables = vec![];

                for nom_sym_info in cil.symbol_crossref_infos {
                    let mut map = ClassMap::new();
                    map.populate(nom_sym_info, server).await?;
                    map.generate_tables(&mut tables);
                }

                Ok(PipelineValues::SymbolTreeTableList(SymbolTreeTableList {
                    tables,
                }))
            }
        }
    }
}
