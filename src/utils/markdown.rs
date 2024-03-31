use std::collections::HashMap;
use std::io::Write;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use comrak::{
    markdown_to_html_with_plugins, ExtensionOptionsBuilder, Options, PluginsBuilder,
    RenderOptionsBuilder, RenderPluginsBuilder,
};
use serde_json5;

use crate::consts::{ALPINE_HIGHLIGHTING_APP, ALPINE_ORIG_CODE_ELM, ATTR_CODEFENCE_EXTRA};
use crate::types::CodeFenceOptions;

// A simple adapter that defers highlighting job to the client side
pub struct JsHighlightAdapter;

impl SyntaxHighlighterAdapter for JsHighlightAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        _lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        html::escape(output, code.as_bytes())
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        mut attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        // Adding HTML classes which are needed by our AlpineJS app
        let classname = " q-need-highlight not-prose p-0";
        if let Some(class) = attributes.get_mut("class") {
            class.push_str(classname)
        } else {
            attributes.insert("class".to_string(), classname.to_string());
        };
        attributes.insert("x-data".to_string(), ALPINE_HIGHLIGHTING_APP.to_string());
        attributes.insert("x-html".to_string(), "highlight()".to_string());
        html::write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        mut attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        // Adding HTML classes which are needed by our AlpineJS app
        tracing::info!("Attributes for code: {:?}", attributes);
        let mut class_names = vec!["q-code"];
        if let Some(info_string) = attributes.get(ATTR_CODEFENCE_EXTRA) {
            tracing::info!("Attempt to parse: {}", info_string);
            let codefence_opts: CodeFenceOptions = serde_json5::from_str(info_string.as_str())
                .inspect_err(|e| tracing::warn!("Failed to parse codefence extra. {e}"))
                .unwrap_or_default();
            if codefence_opts.lines {
                class_names.push("q-with-lineno")
            }
            attributes.insert(
                "data-start-line".to_string(),
                format!("{}", codefence_opts.start_line),
            );
        };
        let extra_class = format!(" {}", class_names.join(" "));
        if let Some(class) = attributes.get_mut("class") {
            class.push_str(&extra_class)
        } else {
            attributes.insert("class".to_string(), extra_class);
        };
        attributes.insert("x-ref".to_string(), ALPINE_ORIG_CODE_ELM.to_string());
        html::write_opening_tag(output, "code", attributes)
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let extension = ExtensionOptionsBuilder::default()
        .table(true)
        .autolink(true)
        .build()
        .unwrap_or_default();
    let render = RenderOptionsBuilder::default()
        .full_info_string(true)
        .build()
        .unwrap_or_default();
    let options = Options {
        extension,
        render,
        ..Default::default()
    };
    let adapter = JsHighlightAdapter;
    let render = RenderPluginsBuilder::default()
        .codefence_syntax_highlighter(Some(&adapter))
        .build()
        .unwrap_or_default();
    let plugins = PluginsBuilder::default()
        .render(render)
        .build()
        .unwrap_or_default();
    markdown_to_html_with_plugins(markdown, &options, &plugins)
}

pub fn make_excerpt(markdown: &str) -> String {
    let mut lines: Vec<&str> = markdown.lines().take(7).collect();
    // Count "code block" marker (```)
    let count: u8 = lines.iter().map(|ln| ln.starts_with("```") as u8).sum();
    // There are odd number of marks
    if count % 2 == 1 {
        // Remove last mark...
        if lines.last().unwrap_or(&"").starts_with("```") {
            lines = lines[..lines.len() - 1].to_vec();
        } else {
            // ...Or add another mark to make sure the number is even
            lines.push("```");
        }
    }
    let reduced = lines.join("\n");
    let html = markdown_to_html(&reduced);
    html + "..."
}
