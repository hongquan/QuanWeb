use lazy_regex::{lazy_regex, regex, Lazy};
use syntect::html::ClassStyle;

#[allow(dead_code)]
pub const DB_NAME: &str = "quanweb";
pub const DEFAULT_PAGE_SIZE: u8 = 10;
pub const STATIC_URL: &str = "/static";
pub const UNCATEGORIZED_URL: &str = "/category/_uncategorized/";
#[allow(dead_code)]
pub const SYNTECT_CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "st-" };
pub const KEY_LANG: &str = "lang";
pub const DEFAULT_LANG: &str = "en";
pub const ALPINE_HIGHLIGHTING_APP: &str = "need_highlight";
pub const ALPINE_ORIG_CODE_ELM: &str = "orig_code";
// Given by comrak
pub const ATTR_CODEFENCE_EXTRA: &str = "data-meta";
// URL allowlist for media embeds (YouTube / asciinema). Each regex captures
// exactly one group: the validated id `^[A-Za-z0-9_-]{6,}$`. Iframes are built
// only from this captured id and fixed literals, never from user attributes.
pub static YT_WATCH_RE: Lazy<regex::Regex> = lazy_regex!(
    r"^https?://(?:www\.)?(?:youtube\.com/watch\?v=|music\.youtube\.com/watch\?v=|youtube\.com/embed/)([A-Za-z0-9_-]{6,})/?$"
);
pub static YT_SHORT_RE: Lazy<regex::Regex> =
    lazy_regex!(r"^https?://youtu\.be/([A-Za-z0-9_-]{6,})/?$");
pub static ASCII_RE: Lazy<regex::Regex> =
    lazy_regex!(r"^https?://asciinema\.org/a/([A-Za-z0-9_-]{6,})/?$");
pub const EMBED_CLASS_YOUTUBE: &str = "q-embed q-embed-youtube";
pub const EMBED_CLASS_ASCII: &str = "q-embed q-embed-asciinema";
// Tracking
pub const MATOMO_URL: &str = "matomo.quan.hoabinh.vn";
pub const MATOMO_SITE_ID: u8 = 1;
// User-agent of the AI agents
pub const AI_AGENT_PATTERNS: [&str; 19] = [
    "ChatGPT-User",
    "MistralAI-User",
    "Gemini-Deep-Research",
    "Claude-User",
    "Perplexity-User",
    "Google-NotebookLM",
    "Google-GeminiNotebook",
    "GPTBot",
    "OAI-SearchBot",
    "ClaudeBot",
    "Anthropic",
    "CohereAI",
    "Bytespider",
    "Amazonbot",
    "CCBot",
    "Google-Extended",
    "PerplexityBot",
    "Applebot-Extended",
    "meta-externalagent",
];
// If AI agents request resources of this pattern, ignore tracking.
