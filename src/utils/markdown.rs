use core::fmt;
use std::borrow::Cow;
use std::collections::HashMap;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use comrak::nodes::NodeValue;
use comrak::options::{Extension, Plugins, Render, RenderPlugins};
use comrak::{Arena, Options, format_html_with_plugins, parse_document};
use minijinja::{Environment, context};
use serde_json5;

use crate::consts::{
    ALPINE_HIGHLIGHTING_APP, ALPINE_ORIG_CODE_ELM, ASCII_RE, ATTR_CODEFENCE_EXTRA,
    EMBED_CLASS_ASCII, EMBED_CLASS_YOUTUBE, YT_SHORT_RE, YT_WATCH_RE,
};
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
    render_markdown(markdown, true)
}

fn render_markdown(markdown: &str, allow_embeds: bool) -> String {
    let extension = Extension::builder().table(true).autolink(true).build();
    // `unsafe_` is required for HtmlInline (embeds) to pass through raw. Only
    // our own validated iframes use that escape hatch; everything else still
    // comes from comrak's own escaping.
    let mut render = Render::builder().full_info_string(true).build();
    render.r#unsafe = allow_embeds;
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
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);
    transform_embeds(root, allow_embeds);
    let mut html = String::new();
    format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();
    html
}

/// Replace image nodes whose URL points to YouTube or asciinema with an embed
/// iframe. The iframe HTML is built only from a validated id, never from
/// user-supplied attributes.
fn transform_embeds(root: comrak::Node, allow_embeds: bool) {
    if !allow_embeds {
        return;
    }
    for node in root.descendants() {
        let embed = {
            let data = node.data.borrow_mut();
            if let NodeValue::Image(ref link) = data.value {
                embed_html(&link.url)
            } else {
                None
            }
        };
        if let Some(html) = embed {
            node.data.borrow_mut().value = NodeValue::HtmlInline(html);
        }
    }
}

fn embed_html(url: &str) -> Option<String> {
    if let Some(caps) = YT_WATCH_RE.captures(url) {
        return Some(youtube_embed(&caps[1]));
    }
    if let Some(caps) = YT_SHORT_RE.captures(url) {
        return Some(youtube_embed(&caps[1]));
    }
    if let Some(caps) = ASCII_RE.captures(url) {
        let id = &caps[1];
        return Some(format!(
            r#"<iframe src="https://asciinema.org/a/{id}/iframe" loading="lazy" class="{EMBED_CLASS_ASCII}" allowfullscreen></iframe>"#
        ));
    }
    None
}

fn youtube_embed(id: &str) -> String {
    format!(
        r#"<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/{id}" title="YouTube video player" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen loading="lazy" class="{EMBED_CLASS_YOUTUBE}"></iframe>"#
    )
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
    let html = render_markdown(&reduced, false);
    html + "…"
}

fn extract_reference_labels(line: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut chars = line.char_indices().peekable();
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
    let html = render_markdown(markdown, true);
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
        assert!(html.ends_with("…"));
    }

    #[test]
    fn test_make_excerpt_reference_links_preserved() {
        let markdown = "Visit [pypi]\n\n[pypi]: https://pypi.org/\n";
        let html = make_excerpt(markdown);
        assert!(html.contains(r#"<a href="https://pypi.org/">pypi</a>"#));
        assert!(html.ends_with("…"));
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
        assert!(html.ends_with("…"));
    }

    #[test]
    fn test_youtube_watch_embed() {
        let html = markdown_to_html("![youtube](https://www.youtube.com/watch?v=dQw4w9WgXcQ)");
        assert!(html.contains(r#"src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ""#));
        assert!(html.contains("q-embed-youtube"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_youtube_short_embed() {
        let html = markdown_to_html("![youtube](https://youtu.be/dQw4w9WgXcQ)");
        assert!(html.contains(r#"src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ""#));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_asciinema_embed() {
        let html = markdown_to_html("![asciinema](https://asciinema.org/a/293140)");
        assert!(html.contains(r#"src="https://asciinema.org/a/293140/iframe""#));
        assert!(html.contains("q-embed-asciinema"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_normal_image_untouched() {
        let html = markdown_to_html("![cat](cat.png)");
        assert!(html.contains(r#"<img src="cat.png" alt="cat""#));
        assert!(!html.contains("<iframe"));
    }

    #[test]
    fn test_evil_embed_urls_rejected() {
        for url in [
            "https://www.youtube.com/watch?v=;alert(1)",
            "https://youtu.be/;alert(1)",
            "https://asciinema.org/a/;alert(1)",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ\" onload=\"alert(1)",
        ] {
            let html = markdown_to_html(&format!("![x]({url})"));
            assert!(
                !html.contains("<iframe"),
                "expected no iframe for {url:?}, got: {html}"
            );
        }
    }

    #[test]
    fn test_excerpt_has_no_iframe() {
        let markdown = "![youtube](https://youtu.be/dQw4w9WgXcQ)";
        let html = make_excerpt(markdown);
        assert!(
            !html.contains("<iframe"),
            "excerpt must not contain an iframe, got: {html}"
        );
        assert!(html.ends_with("…"));
    }
}
