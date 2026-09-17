//! AI Chatbot Tracking for Matomo.
//! Matomo Tracking API: https://developer.matomo.org/api-reference/tracking-api#tracking-bots

use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::consts::{AI_AGENT_PATTERNS, MATOMO_SITE_ID, MATOMO_URL, URL_IGNORE_PATTERN};

/// Target of a chatbot visit (page or download).
#[derive(Debug, Clone, Serialize)]
pub enum ChatbotTarget {
    Page(String),
    Download(String),
}

/// Unique identifier for a chatbot visit (ULID).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AIChatbotVisitId(pub ulid::Ulid);

impl AIChatbotVisitId {
    pub fn new() -> Self {
        Self(ulid::Ulid::generate())
    }
}

impl Serialize for AIChatbotVisitId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

impl Display for AIChatbotVisitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Before-response event: contains what's known before the handler runs.
#[derive(Debug, Clone, Serialize)]
pub struct AIChatbotPartial {
    pub visit_id: AIChatbotVisitId,
    pub target: ChatbotTarget,
    pub user_agent: String,
    /// ISO-8601 UTC timestamp when the visit started.
    pub cdt: DateTime<Utc>,
}

/// After-response event: contains what's known after the handler completes.
#[derive(Debug, Clone, Serialize)]
pub struct AIChatbotVisitResponse {
    pub visit_id: AIChatbotVisitId,
    pub http_status: u16,
    pub bw_bytes: u32,
    /// Server-side processing time in milliseconds.
    pub pf_srv: u64,
}

/// Unified event type for channel communication.
#[derive(Debug, Clone)]
pub enum AIChatbotEvent {
    Request(AIChatbotPartial),
    Response(AIChatbotVisitResponse),
}

/// Check whether the user-agent string matches any known AI agent pattern.
pub fn is_ai_agent(user_agent: &str) -> bool {
    AI_AGENT_PATTERNS
        .iter()
        .any(|pattern| user_agent.contains(pattern))
}

/// Check whether the request path matches the ignored URL pattern.
pub fn is_ignored_url(path: &str) -> bool {
    static IGNORE_RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(URL_IGNORE_PATTERN).unwrap());
    IGNORE_RE.is_match(path)
}

/// Classify a request URI into a ChatbotTarget with the full URL.
pub fn classify_target(scheme: &str, host: &str, uri: &http::Uri) -> (String, ChatbotTarget) {
    let full_url = format!(
        "{}://{}{}",
        scheme,
        host,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );
    let target = if is_download_url(uri.path()) {
        ChatbotTarget::Download(full_url.clone())
    } else {
        ChatbotTarget::Page(full_url.clone())
    };
    (full_url, target)
}

/// Check whether a URI path looks like a downloadable file.
pub fn is_download_url(path: &str) -> bool {
    const DOWNLOAD_EXTS: &[&str] = &[
        "pdf", "docx", "doc", "zip", "rar", "7z", "tar", "gz", "xls", "xlsx", "ppt", "pptx", "odt",
        "ods", "odp", "rtf", "csv", "epub", "mobi",
    ];
    let ext = path.split('.').last().unwrap_or("");
    DOWNLOAD_EXTS.iter().any(|e| ext.eq_ignore_ascii_case(e))
}

/// Build the Matomo tracking URL for a bot visit.
pub fn build_matomo_url(target: &ChatbotTarget, user_agent: &str, cdt: &DateTime<Utc>) -> String {
    let page_url = match target {
        ChatbotTarget::Page(u) => u.as_str(),
        ChatbotTarget::Download(u) => u.as_str(),
    };
    let cdt = cdt.format("%Y-%m-%dT%H:%M:%S").to_string();
    reqwest::Url::parse_with_params(
        &format!("https://{MATOMO_URL}/matomo.php"),
        &[
            ("idsite", MATOMO_SITE_ID.to_string()),
            ("rec", "1".to_string()),
            ("bots", "1".to_string()),
            ("url", page_url.to_string()),
            ("ua", user_agent.to_string()),
            ("cdt", cdt),
        ],
    )
    .expect("static Matomo base URL is always parseable")
    .to_string()
}

/// Tower middleware that detects AI chatbot visits and sends tracking events to a channel.
pub async fn ai_chatbot_tracking_middleware(
    axum::extract::State(tx): axum::extract::State<
        std::sync::Arc<tokio::sync::mpsc::Sender<AIChatbotEvent>>,
    >,
    req: http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Only track GET requests
    if req.method() != http::Method::GET {
        return next.run(req).await;
    }

    // Check if URL is ignored
    let path = req.uri().path();
    if is_ignored_url(path) {
        return next.run(req).await;
    }

    // Check if User-Agent matches AI agent patterns
    let bot_ua = match req
        .headers()
        .get(http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
    {
        Some(ua) if is_ai_agent(ua) => ua.to_string(),
        _ => return next.run(req).await,
    };

    // Build the target URL and classify (page vs download)
    let scheme = req.uri().scheme().map(|s| s.as_str()).unwrap_or("https");
    let host = req
        .headers()
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let (_full_url, target) = classify_target(scheme, host, req.uri());

    // Generate visit ID and send partial event
    let visit_id = AIChatbotVisitId::new();
    let partial = AIChatbotPartial {
        visit_id: visit_id.clone(),
        target,
        user_agent: bot_ua,
        cdt: Utc::now(),
    };
    if tx.try_send(AIChatbotEvent::Request(partial)).is_err() {
        tracing::warn!(
            "Chatbot tracking channel full, dropping event for visit {}",
            visit_id
        );
    }

    // Run the handler and send response event
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let body_size = response
        .headers()
        .get(http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    if tx
        .try_send(AIChatbotEvent::Response(AIChatbotVisitResponse {
            visit_id: visit_id.clone(),
            http_status: status,
            bw_bytes: body_size,
            pf_srv: duration,
        }))
        .is_err()
    {
        tracing::warn!(
            "Chatbot tracking channel full, dropping response for visit {}",
            visit_id
        );
    }

    response
}
