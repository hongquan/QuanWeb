use std::collections::HashMap;
use std::io::Write;

use htmlize::escape_text;
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
use syntect::html::ClassedHTMLGenerator;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

use crate::consts::{SYNTECT_CLASS_STYLE, ALPINE_HIGHLIGHTING_APP, ALPINE_ORIG_CODE_ELM};

pub struct CssSyntectAdapter {
    syntax_set: SyntaxSet,
}

#[allow(dead_code)]
impl CssSyntectAdapter {
    pub fn new() -> Self {
        Self {
            syntax_set: two_face::syntax::extra_newlines(),
        }
    }

    fn highlight_html(
        &self,
        code: &str,
        syntax: &SyntaxReference,
    ) -> Result<String, syntect::Error> {
        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            SYNTECT_CLASS_STYLE,
        );
        for line in LinesWithEndings::from(code) {
            html_generator.parse_html_for_line_which_includes_newline(line)?;
        }
        Ok(html_generator.finalize())
    }
}

impl SyntaxHighlighterAdapter for CssSyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> std::io::Result<()> {
        let fallback_syntax = "Plain Text";
        let lang: &str = match lang {
            Some(l) if !l.is_empty() => l,
            _ => fallback_syntax,
        };
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| {
                self.syntax_set
                    .find_syntax_by_first_line(code)
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            });

        match self.highlight_html(code, syntax) {
            Ok(highlighted_code) => output.write_all(highlighted_code.as_bytes()),
            Err(_) => output.write_all(code.as_bytes()),
        }
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        html::write_opening_tag(output, "pre", attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        html::write_opening_tag(output, "code", attributes)
    }
}

// A simple adapter that defers highlighting job to the client side
pub struct JSHighlightSyntectAdapter;

impl SyntaxHighlighterAdapter for JSHighlightSyntectAdapter {
    fn write_highlighted(
            &self,
            output: &mut dyn Write,
            _lang: Option<&str>,
            code: &str,
        ) -> std::io::Result<()> {
        let code = escape_text(code);
        output.write_all(code.as_bytes())
    }

    fn write_pre_tag(
            &self,
            output: &mut dyn Write,
            mut attributes: HashMap<String, String>,
        ) -> std::io::Result<()> {
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
        let classname = " q-code";
        if let Some(class) = attributes.get_mut("class") {
            class.push_str(classname)
        } else {
            attributes.insert("class".to_string(), classname.to_string());
        };
        attributes.insert("x-ref".to_string(), ALPINE_ORIG_CODE_ELM.to_string());
        html::write_opening_tag(output, "code", attributes)
    }
}

pub fn markdown_to_html(markdown: &str) -> String {
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    let adapter = JSHighlightSyntectAdapter;
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
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
