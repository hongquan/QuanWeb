use core::fmt;
use std::borrow::Cow;
use std::collections::HashMap;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use comrak::options::{Extension, Plugins, Render, RenderPlugins};
use comrak::{Options, markdown_to_html_with_plugins};
use minijinja::{Environment, context};
use serde_json5;

use crate::consts::{ALPINE_HIGHLIGHTING_APP, ALPINE_ORIG_CODE_ELM, ATTR_CODEFENCE_EXTRA};
use crate::errors::PageError;
use crate::types::CodeFenceOptions;
use crate::utils::html::render_with;

// A simple adapter that defers highlighting job to the client side
pub struct JsHighlightAdapter;

impl SyntaxHighlighterAdapter for JsHighlightAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        _lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        html::escape(output, code)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        // Adding HTML classes which are needed by our AlpineJS app
        let classname = "q-need-highlight not-prose p-0";
        if let Some(class) = attributes.remove("class") {
            attributes.insert("class", Cow::from(format!("{class} {classname}")));
        } else {
            attributes.insert("class", Cow::from(classname));
        };
        attributes.insert("x-data", Cow::from(ALPINE_HIGHLIGHTING_APP));
        attributes.insert("x-html", Cow::from("highlight()"));
        html::write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        // Adding HTML classes which are needed by our AlpineJS app
        tracing::info!("Attributes for code: {:?}", attributes);
        let mut class_names = vec!["q-code"];
        if let Some(info_string) = attributes.get(ATTR_CODEFENCE_EXTRA) {
            tracing::info!("Attempt to parse: {}", info_string);
            let codefence_opts: CodeFenceOptions = serde_json5::from_str(info_string.as_ref())
                .inspect_err(|e| tracing::warn!("Failed to parse codefence extra. {e}"))
                .unwrap_or_default();
            if codefence_opts.lines {
                class_names.push("q-with-lineno")
            }
            attributes.insert(
                "data-start-line",
                Cow::from(format!("{}", codefence_opts.start_line)),
            );
        };
        let extra_class = format!(" {}", class_names.join(" "));
        if let Some(class) = attributes.remove("class") {
            attributes.insert("class", Cow::from(format!("{class} {extra_class}")));
        } else {
            attributes.insert("class", Cow::from(extra_class));
        };
        attributes.insert("x-ref", Cow::from(ALPINE_ORIG_CODE_ELM));
        html::write_opening_tag(output, "code", attributes)
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let extension = Extension::builder().table(true).autolink(true).build();
    let render = Render::builder().full_info_string(true).build();
    let options = Options {
        extension,
        render,
        ..Default::default()
    };
    let adapter = JsHighlightAdapter;
    let render = RenderPlugins::builder()
        .codefence_syntax_highlighter(&adapter)
        .build();
    let plugins = Plugins::builder().render(render).build();
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

// Convert markdown to full HTML document (enough markups), suitable to be
// shown in an iframe.
pub fn markdown_to_html_document(markdown: &str, engine: Environment) -> Result<String, PageError> {
    let extension = Extension::builder().table(true).autolink(true).build();
    let render = Render::builder().full_info_string(true).build();
    let options = Options {
        extension,
        render,
        ..Default::default()
    };
    let adapter = JsHighlightAdapter;
    let render = RenderPlugins::builder()
        .codefence_syntax_highlighter(&adapter)
        .build();
    let plugins = Plugins::builder().render(render).build();
    let html = markdown_to_html_with_plugins(markdown, &options, &plugins);
    let vcontext = context! {
        content => html,
    };
    render_with("mini-preview.jinja", vcontext, engine)
}
