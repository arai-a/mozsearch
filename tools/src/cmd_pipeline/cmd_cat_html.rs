use async_trait::async_trait;
use clap::Args;
use lol_html::{
    element, html_content::ContentType, rewrite_str, HtmlRewriter, RewriteStrSettings, Settings,
};
use std::{cell::Cell, rc::Rc};

use super::interface::{PipelineCommand, PipelineValues, TextFile};
use crate::abstract_server::{AbstractServer, HtmlFileRoot, Result};

/// Dump the contents of a HTML file for a (source) file or rendered directory
/// listing from disk in its entirety, applying minimal normalization to
/// compensate for datestamps or specific revision data.
///
/// Intended exclusively for regression testing and in particular to provide
/// coverage for directory listings and semi-generated files like "help.html"
/// and "settings.html" where we want to audit changes to the file in their
/// entirety but we don't want to have N copies of the searchfox HTML super
/// structure.  We would expect to use this command for at most a few source
/// files to validate where the source listing joins to the HTML super structure
/// but would expect to use "show-html" for most "checks" so that they can be
/// much more targeted (than having potentially massive diffs that are
/// constantly including the entirety of a generated page with many irrelevant
/// changes to the specific purpose of the check).
///
/// Differs from show-html which is about excerpting source lines and which has
/// a separate "prod-filter" helper for production "checks".
#[derive(Debug, Args)]
pub struct CatHtml {
    /// Tree-relative source file path or directory.
    #[clap(value_parser)]
    file: String,

    /// Is this a directory's HTML we want (instead of a source file)?
    #[clap(short, long, action)]
    dir: bool,

    /// Is this a template's HTML we want?
    #[clap(short, long, action)]
    template: bool,

    /// Use a CSS selector to limit the returned portion of the document.  This
    /// can be useful to focus a test and make diffs easier to understand.
    #[clap(short, long, value_parser)]
    select: Option<String>,
}

#[derive(Debug)]
pub struct CatHtmlCommand {
    pub args: CatHtml,
}

// HTML normalization of our expected entire HTML files:
// - "This page was generated by Searchfox DATETIME": We wrap the
//   datetime in a spam with class "pretty-date" and attribute with key
//   "data-datetime".  We currently normalize by replacing it with a span
//   `<span>NORMALIZED</span>` which loses the extra attributes but we don't
//   care about that level of fidelity.
fn norm_html_file(s: String) -> String {
    let element_content_handlers = vec![element!(r#"span.pretty-date"#, |el| {
        el.replace("<span>NORMALIZED</span>", ContentType::Html);
        Ok(())
    })];

    rewrite_str(
        &s,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )
    .unwrap()
}

fn extract_html_snippet(html_str: String, selector: &str) -> String {
    let mut excerpts = vec![];

    let suppressing = Rc::new(Cell::new(true));
    let sink_suppressing = suppressing.clone();

    let mut buf = vec![];

    let synthetic_closing = Rc::new(Cell::new(None));
    let sink_closing = synthetic_closing.clone();

    let mut rewrite = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!(selector, move |el| {
                suppressing.set(false);
                let end_suppress = suppressing.clone();
                let end_closing = synthetic_closing.clone();
                el.on_end_tag(move |end| {
                    end_closing.set(Some(format!("</{}>", end.name())));
                    end_suppress.set(true);
                    Ok(())
                })?;
                Ok(())
            })],
            ..Settings::default()
        },
        |c: &[u8]| {
            if sink_suppressing.get() {
                if let Some(closing) = sink_closing.take() {
                    buf.extend_from_slice(closing.as_bytes());
                }
                // Flush if this was apparently a transition from accumulating
                // into our buffer.
                if !buf.is_empty() {
                    excerpts.push(String::from_utf8_lossy(&buf).to_string());
                    buf.clear();
                }
                return;
            }

            buf.extend_from_slice(c);
        },
    );

    rewrite.write(html_str.as_bytes()).unwrap();
    rewrite.end().unwrap();

    excerpts.join("\n")
}

#[async_trait]
impl PipelineCommand for CatHtmlCommand {
    async fn execute(
        &self,
        server: &(dyn AbstractServer + Send + Sync),
        _input: PipelineValues,
    ) -> Result<PipelineValues> {
        let root = if self.args.dir {
            HtmlFileRoot::FormattedDir
        } else if self.args.template {
            HtmlFileRoot::FormattedTemplate
        } else {
            HtmlFileRoot::FormattedFile
        };
        let mut html_str = server.fetch_html(root, &self.args.file).await?;

        if let Some(selector) = &self.args.select {
            html_str = extract_html_snippet(html_str, selector);
        }

        Ok(PipelineValues::TextFile(TextFile {
            mime_type: "text/html".to_string(),
            contents: norm_html_file(html_str),
        }))
    }
}
