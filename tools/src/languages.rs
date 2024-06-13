use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct LanguageSpec {
    pub reserved_words: HashMap<String, String>,
    pub hash_comment: bool,
    // In JS, private symbols are now a thing.
    pub hash_identifier: bool,
    pub c_style_comments: bool,
    pub backtick_strings: bool,
    pub regexp_literals: bool,
    pub triple_quote_literals: bool,
    pub c_preprocessor: bool,
    // Rust is mostly C-like, with a couple of differences.
    pub rust_tweaks: bool,
    pub cxx14_digit_separators: bool,
    pub markdown_slug: &'static str,
}

pub const SYN_RESERVED_CLASS: &'static str = "syn_reserved";
pub const SYN_CONTEXTUAL_KEYWORD_CLASS: &'static str = "syn_contextual";

fn make_reserved(v: &[&str]) -> HashMap<String, String> {
    let mut reserved_words = HashMap::new();
    for word in v {
        reserved_words.insert(word.to_string(), SYN_RESERVED_CLASS.into());
    }
    reserved_words
}

fn make_reserved_with_contextual(
    reserved: &[&str], contextual: &[&str]) -> HashMap<String, String>
{
    let mut reserved_words = HashMap::new();
    for word in reserved {
        reserved_words.insert(word.to_string(), SYN_RESERVED_CLASS.into());
    }
    for word in contextual {
        reserved_words.insert(word.to_string(), SYN_CONTEXTUAL_KEYWORD_CLASS.into());
    }
    reserved_words
}

static RESERVED_WORDS_JS: &'static [&'static str] = &[
    "abstract",
    "else",
    "instanceof",
    "super",
    "boolean",
    "enum",
    "int",
    "switch",
    "break",
    "export",
    "synchronized",
    "byte",
    "extends",
    "this",
    "case",
    "false",
    "long",
    "throw",
    "catch",
    "final",
    "native",
    "throws",
    "char",
    "finally",
    "new",
    "transient",
    "class",
    "float",
    "null",
    "true",
    "const",
    "for",
    "try",
    "continue",
    "function",
    "typeof",
    "debugger",
    "goto",
    "var",
    "default",
    "if",
    "void",
    "delete",
    "return",
    "volatile",
    "do",
    "import",
    "short",
    "while",
    "double",
    "in",
    "with",
];

static CONTEXTUAL_KEYWORDS_JS: &'static [&'static str] = &[
    // Keyword in async context.
    "await",

    // Keywrod in generator.
    "yield",

    // Keyword in strict.
    "let",
    "static",
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "public",

    // Certain Syntactic production.
    "as",
    "async",
    "from",
    "get",
    "meta",
    "of",
    "set",
    "target",
];

static RESERVED_WORDS_CPP: &'static [&'static str] = &[
    "alignas",
    "alignof",
    "and",
    "and_eq",
    "asm",
    "atomic_cancel",
    "atomic_commit",
    "atomic_noexcept",
    "auto",
    "bitand",
    "bitor",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "char16_t",
    "char32_t",
    "class",
    "compl",
    "concept",
    "const",
    "constexpr",
    "const_cast",
    "continue",
    "decltype",
    "default",
    "delete",
    "do",
    "double",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "int",
    "import",
    "long",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "not_eq",
    "nullptr",
    // Don't mark "operator" as a keyword so that people can click
    // on it.
    //"operator",
    "or",
    "or_eq",
    "private",
    "protected",
    "public",
    "register",
    "reinterpret_cast",
    "requires",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "synchronized",
    "template",
    "this",
    "thread_local",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "wchar_t",
    "while",
    "xor",
    "xor_eq",
    "#if",
    "#ifdef",
    "#ifndef",
    "#else",
    "#elif",
    "#endif",
    "#define",
    "#undef",
    "#include",
    "#error",
    "defined",
];

static CONTEXTUAL_KEYWORDS_CPP: &'static [&'static str] = &[
    "final",
    "import",
    "module",
    "override",
];

static RESERVED_WORDS_AIDL: &'static [&'static str] = &[
    "parcelable",
    "import",
    "package",
    "in",
    "out",
    "inout",
    "cpp_header",
    "const",
    "true",
    "false",
    "interface",
    "oneway",
    "enum",
    "union",
];

// From 'reserved' in ipc/ipdl/ipdl/parser.py
static RESERVED_WORDS_IPDL: &'static [&'static str] = &[
    "async",
    "both",
    "child",
    "class",
    "compress",
    "compressall",
    "from",
    "include",
    "intr",
    "manager",
    "manages",
    "namespace",
    "nested",
    "nullable",
    "or",
    "parent",
    "prio",
    "protocol",
    "refcounted",
    "moveonly",
    "returns",
    "struct",
    "sync",
    "union",
    "UniquePtr",
    "upto",
    "using",
    "verify",
];

static RESERVED_WORDS_IDL: &'static [&'static str] = &[
    "cenum",
    "const",
    "interface",
    "in",
    "inout",
    "out",
    "attribute",
    "raises",
    "readonly",
    "native",
    "typedef",
    "webidl",
    "array",
    "shared",
    "iid_is",
    "size_is",
    "retval",
    "boolean",
    "void",
    "octet",
    "short",
    "long",
    "unsigned",
    "float",
    "double",
    "char",
    "string",
    "wchar",
    "wstring",
    "nsid",
    "AUTF8String",
    "ACString",
    "AString",
    "jsval",
    "Promise",
    "ptr",
    "ref",
    "uuid",
    "scriptable",
    "builtinclass",
    "function",
    "noscript",
    "deprecated",
    "object",
    "main_process_scriptable_only",
    "binaryname",
    "notxpcom",
    "symbol",
    "implicit_jscontext",
    "nostdcall",
    "must_use",
    "infallible",
    "can_run_script",
];

static RESERVED_WORDS_WEBIDL: &'static [&'static str] = &[
    "module",
    "interface",
    "partial",
    "dictionary",
    "exception",
    "enum",
    "callback",
    "typedef",
    "implements",
    "const",
    "null",
    "true",
    "false",
    "serializer",
    "stringifier",
    "jsonifier",
    "unrestricted",
    "attribute",
    "readonly",
    "inherit",
    "static",
    "getter",
    "setter",
    "creator",
    "deleter",
    "legacycaller",
    "optional",
    "Date",
    "DOMString",
    "ByteString",
    "USVString",
    "any",
    "boolean",
    "byte",
    "double",
    "float",
    "long",
    "object",
    "octet",
    "Promise",
    "required",
    "sequence",
    "MozMap",
    "short",
    "unsigned",
    "void",
    "ArrayBuffer",
    "SharedArrayBuffer",
    "or",
    "maplike",
    "setlike",
    "iterable",
    "Exposed",
    "ChromeOnly",
    "ChromeConstructor",
    "Pref",
    "Func",
    "AvailableIn",
    "CheckAnyPermissions",
    "CheckAllPermissions",
    "JSImplementation",
    "HeaderFile",
    "NavigatorProperty",
    "AvailableIn",
    "Func",
    "CheckAnyPermissions",
    "CheckAllPermissions",
    "Deprecated",
    "NeedResolve",
    "OverrideBuiltins",
    "ChromeOnly",
    "Unforgeable",
    "UnsafeInPrerendering",
    "LegacyEventInit",
    "ProbablyShortLivingObject",
    "ArrayClass",
    "Clamp",
    "Constructor",
    "EnforceRange",
    "ExceptionClass",
    "Exposed",
    "ImplicitThis",
    "Global",
    "PrimaryGlobal",
    "LegacyArrayClass",
    "LegacyUnenumerableNamedProperties",
    "LenientSetter",
    "LenientThis",
    "NamedConstructor",
    "NewObject",
    "NoInterfaceObject",
    "OverrideBuiltins",
    "PutForwards",
    "Replaceable",
    "SameObject",
    "SecureContext",
    "Throws",
    "TreatNonObjectAsNull",
    "TreatNullAs",
    "Unforgeable",
    "Unscopable",
];

static RESERVED_WORDS_PYTHON: &'static [&'static str] = &[
    "and", "del", "from", "not", "while", "as", "elif", "global", "or", "with", "assert", "else",
    "if", "pass", "yield", "break", "except", "import", "print", "class", "exec", "in", "raise",
    "continue", "finally", "is", "return", "def", "for", "lambda", "try",
];

// List of Rust reserved words pulled from
// https://github.com/rust-lang/rust/blob/master/src/libsyntax/symbol.rs
static RESERVED_WORDS_RUST: &'static [&'static str] = &[
    "as", "box", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "abstract", "alignof", "become", "do", "final", "macro", "offsetof",
    "override", "priv", "proc", "pure", "sizeof", "typeof", "unsized", "virtual", "yield",
    "default", "union",
];

static RESERVED_WORDS_JAVA: &'static [&'static str] = &[
    "abstract",
    "continue",
    "for",
    "new",
    "switch",
    "assert",
    "default",
    "goto",
    "package",
    "synchronized",
    "boolean",
    "do",
    "if",
    "private",
    "this",
    "break",
    "double",
    "implements",
    "protected",
    "throw",
    "byte",
    "else",
    "import",
    "public",
    "throws",
    "case",
    "enum",
    "instanceof",
    "return",
    "transient",
    "catch",
    "extends",
    "int",
    "short",
    "try",
    "char",
    "final",
    "interface",
    "static",
    "void",
    "class",
    "finally",
    "long",
    "strictfp",
    "volatile",
    "const",
    "float",
    "native",
    "super",
    "while",
    "null",
    "true",
    "false",
];

// http://kotlinlang.org/docs/reference/keyword-reference.html
static RESERVED_WORDS_KOTLIN: &'static [&'static str] = &[
    "as",
    "as?",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "!in",
    "interface",
    "is",
    "!is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "val",
    "var",
    "when",
    "while",
    "by",
    "catch",
    "constructor",
    "delegate",
    "dynamic",
    "field",
    "file",
    "finally",
    "get",
    "import",
    "init",
    "param",
    "property",
    "receiver",
    "set",
    "setparam",
    "where",
    "actual",
    "abstract",
    "annotation",
    "companion",
    "const",
    "crossinline",
    "data",
    "enum",
    "expect",
    "external",
    "final",
    "infix",
    "inline",
    "inner",
    "internal",
    "lateinit",
    "noinline",
    "open",
    "operator",
    "out",
    "override",
    "private",
    "protected",
    "public",
    "reified",
    "sealed",
    "suspend",
    "tailrec",
    "vararg",
    "field",
    "it",
];

lazy_static! {
    static ref JS_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved_with_contextual(&*RESERVED_WORDS_JS,
                                                      &*CONTEXTUAL_KEYWORDS_JS),
        hash_identifier: true,
        c_style_comments: true,
        backtick_strings: true,
        regexp_literals: true,
        markdown_slug: "js",
        .. LanguageSpec::default()
    };

    static ref HTML_SPEC : LanguageSpec = LanguageSpec {
        markdown_slug: "html",
        .. JS_SPEC.clone()
    };

    static ref CPP_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved_with_contextual(&*RESERVED_WORDS_CPP,
                                                      &*CONTEXTUAL_KEYWORDS_CPP),
        c_style_comments: true,
        c_preprocessor: true,
        cxx14_digit_separators: true,
        markdown_slug: "cpp",
        .. LanguageSpec::default()
    };

    static ref AIDL_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_AIDL),
        c_style_comments: true,
        .. LanguageSpec::default()
    };

    static ref IPDL_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_IPDL),
        c_style_comments: true,
        .. LanguageSpec::default()
    };

    static ref IDL_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_IDL),
        c_style_comments: true,
        .. LanguageSpec::default()
    };

    static ref WEBIDL_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_WEBIDL),
        c_style_comments: true,
        .. LanguageSpec::default()
    };

    static ref PYTHON_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_PYTHON),
        hash_comment: true,
        triple_quote_literals: true,
        markdown_slug: "py",
        .. LanguageSpec::default()
    };

    static ref RUST_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_RUST),
        hash_comment: true, // for now, for attributes
        c_style_comments: true,
        rust_tweaks: true,
        markdown_slug: "rust",
        .. LanguageSpec::default()
    };

    static ref JAVA_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_JAVA),
        c_style_comments: true,
        markdown_slug: "java",
        .. LanguageSpec::default()
    };

    static ref KOTLIN_SPEC : LanguageSpec = LanguageSpec {
        reserved_words: make_reserved(&*RESERVED_WORDS_KOTLIN),
        c_style_comments: true,
        .. LanguageSpec::default()
    };
}

#[derive(Debug)]
pub enum FormatAs {
    FormatCLike(&'static LanguageSpec),
    FormatTagLike(&'static LanguageSpec),
    CSS,
    Plain,
    Binary,
}

pub fn select_formatting(filename: &str) -> FormatAs {
    let ext = match Path::new(filename).extension() {
        Some(ext) => ext.to_str().unwrap(),
        None => "",
    };
    match ext {
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hxx" | "hpp" | "inc" | "mm" | "m" => {
            FormatAs::FormatCLike(&*CPP_SPEC)
        }
        "aidl" => FormatAs::FormatCLike(&*AIDL_SPEC),
        "ipdl" | "ipdlh" => FormatAs::FormatCLike(&*IPDL_SPEC),
        "idl" => FormatAs::FormatCLike(&*IDL_SPEC),
        "webidl" => FormatAs::FormatCLike(&*WEBIDL_SPEC),
        "js" | "jsm" | "json" | "mjs" | "sjs" => FormatAs::FormatCLike(&*JS_SPEC),
        "py" | "build" | "configure" => FormatAs::FormatCLike(&*PYTHON_SPEC),
        "rs" => FormatAs::FormatCLike(&*RUST_SPEC),
        "java" => FormatAs::FormatCLike(&*JAVA_SPEC),
        "kt" => FormatAs::FormatCLike(&*KOTLIN_SPEC),

        "html" | "htm" | "xhtml" | "xht" | "xml" | "xul" => FormatAs::FormatTagLike(&*HTML_SPEC),

        "css" => FormatAs::CSS,

        // Keep this list in sync with the binary types list in nginx-setup.py
        "ogg" | "ttf" | "xpi" | "png" | "bcmap" | "gif" | "ogv" | "jpg" | "jpeg" | "bmp"
        | "icns" | "ico" | "mp4" | "sqlite" | "jar" | "webm" | "webp" | "woff" | "class"
        | "m4s" | "mgif" | "wav" | "opus" | "mp3" | "otf" => FormatAs::Binary,

        _ => FormatAs::Plain,
    }
}
