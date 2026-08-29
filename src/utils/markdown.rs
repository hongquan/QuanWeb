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
    let mut content_lines: Vec<&str> = markdown.lines().take(7).collect();
    // Collect any link reference definitions used by the excerpted lines.
    let mut definitions = vec![""];
    for ln in content_lines.iter() {
        for label in extract_reference_labels(ln) {
            if let Some(def) = find_link_definition(markdown, &label) {
                if !definitions.contains(&def) {
                    definitions.push(def);
                }
            }
        }
    }
    // Count "code block" marker (```)
    let count: u8 = content_lines
        .iter()
        .map(|ln| ln.starts_with("```") as u8)
        .sum();
    // There are odd number of marks
    if count % 2 == 1 {
        // Remove last mark...
        if content_lines.last().unwrap_or(&"").starts_with("```") {
            content_lines = content_lines[..content_lines.len() - 1].to_vec();
        } else {
            // ...Or add another mark to make sure the number is even
            content_lines.push("```");
        }
    }
    content_lines.extend(definitions);
    let reduced = content_lines.join("\n");
    let html = markdown_to_html(&reduced);
    html + "…"
}

fn extract_reference_labels(line: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut chars = line.chars().enumerate().peekable();
    while let Some((idx, ch)) = chars.next() {
        if ch == '[' {
            let start = idx;
            let mut end = None;
            while let Some((j, c)) = chars.peek() {
                if *c == ']' {
                    end = Some(*j);
                    chars.next();
                    break;
                }
                if *c == '[' || *c == '\n' {
                    break;
                }
                chars.next();
            }
            if let Some(e) = end {
                let label = &line[start + 1..e];
                let label = label.to_lowercase();
                if !label.is_empty() && !label.contains('[') && !label.contains(']') {
                    labels.push(label);
                }
            }
        }
    }
    labels
}

fn find_link_definition<'a>(markdown: &'a str, label: &str) -> Option<&'a str> {
    let normalized_label = label.to_lowercase();
    for def in markdown.lines() {
        if let Some(stripped) = def.strip_prefix('[') {
            if let Some(close_idx) = stripped.find("]:") {
                let def_label = &stripped[..close_idx];
                if def_label.to_lowercase() == normalized_label {
                    return Some(def);
                }
            }
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_excerpt_plain() {
        let markdown = "Hello world\n\nMore content here\nand even more.";
        let html = make_excerpt(markdown);
        assert!(html.contains("<p>Hello world</p>"));
        assert!(html.ends_with("..."));
    }

    #[test]
    fn test_make_excerpt_reference_links_preserved() {
        let markdown = "Visit [pypi]\n\n[pypi]: https://pypi.org/\n";
        let html = make_excerpt(markdown);
        assert!(html.contains(r#"<a href="https://pypi.org/">pypi</a>"#));
        assert!(html.ends_with("..."));
        assert!(!html.contains("[pypi]:"));
    }

    #[test]
    fn test_make_excerpt_reference_definition_not_in_excerpt() {
        let markdown = "Visit [pypi]\n\nSome other line\n\n[pypi]: https://pypi.org/\n";
        let html = make_excerpt(markdown);
        assert!(html.contains(r#"<a href="https://pypi.org/">pypi</a>"#));
    }

    #[test]
    fn test_make_excerpt_multiple_reference_links() {
        let markdown = "Visit [pypi] and [crates]\n\n[pypi]: https://pypi.org/\n[crates]: https://crates.io/\n";
        let html = make_excerpt(markdown);
        assert!(html.contains(r#"<a href="https://pypi.org/">pypi</a>"#));
        assert!(html.contains(r#"<a href="https://crates.io/">crates</a>"#));
    }

    #[test]
    fn test_make_excerpt_code_block_closed() {
        let markdown = "```rust\nlet x = 1;\n```\nFooter";
        let html = make_excerpt(markdown);
        assert!(html.contains("<pre"));
        assert!(html.contains("</pre>"));
        assert!(html.ends_with("..."));
    }
}
