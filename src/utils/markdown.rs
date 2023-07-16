use comrak::{markdown_to_html_with_plugins, ComrakPlugins, ComrakOptions};
use comrak::plugins::syntect::SyntectAdapter;

use crate::consts::SYNTECT_THEME;

pub fn markdown_to_html(markdown: &str) -> String {
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    let adapter = SyntectAdapter::new(SYNTECT_THEME);
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
