//! AI Chatbot Tracking for Matomo.
//! Matomo Tracking API: https://developer.matomo.org/api-reference/tracking-api#tracking-bots

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use chrono::{DateTime, Utc};
use http::Request;
use http::header::USER_AGENT;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::consts::{AI_AGENT_PATTERNS, MATOMO_SITE_ID, MATOMO_URL, URL_IGNORE_PATTERN};

const PENDING_LIMIT: usize = 4096;

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

#[derive(Debug, Clone)]
pub struct PendingVisit {
    visit_id: AIChatbotVisitId,
    target: ChatbotTarget,
    user_agent: String,
    cdt: DateTime<Utc>,
    inserted_at: DateTime<Utc>,
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
    static IGNORE_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(URL_IGNORE_PATTERN).unwrap());
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
pub fn build_matomo_url(
    target: &ChatbotTarget,
    user_agent: &str,
    cdt: &DateTime<Utc>,
    http_status: u16,
    bw_bytes: u32,
    pf_srv: u64,
) -> String {
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
            ("recMode", "1".to_string()),
            ("url", page_url.to_string()),
            ("ua", user_agent.to_string()),
            ("cdt", cdt),
            ("h", http_status.to_string()),
            ("bw_bytes", bw_bytes.to_string()),
            ("pf_srv", pf_srv.to_string()),
        ],
    )
    .expect("static Matomo base URL is always parseable")
    .to_string()
}

/// Tower middleware that detects AI chatbot visits and sends tracking events to a channel.
pub async fn ai_chatbot_tracking_middleware(
    State(tx): State<Arc<mpsc::Sender<AIChatbotEvent>>>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Response {
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
    let bot_ua = match req.headers().get(USER_AGENT).and_then(|v| v.to_str().ok()) {
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
    let (full_url, target) = classify_target(scheme, host, req.uri());

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

    let message = AIChatbotEvent::Response(AIChatbotVisitResponse {
        visit_id: visit_id.clone(),
        http_status: status,
        bw_bytes: body_size,
        pf_srv: duration,
    });
    if let Ok(_v) = tx.try_send(message) {
        tracing::info!("Tracked a AI Chatbot visit at {}", full_url)
    } else {
        tracing::warn!(
            "Chatbot tracking channel full, dropping response for visit {}",
            visit_id
        );
    };

    response
}

pub async fn handle_ai_chatbot_event(
    client: &reqwest::Client,
    pending: &mut HashMap<AIChatbotVisitId, PendingVisit>,
    event: AIChatbotEvent,
) {
    match event {
        AIChatbotEvent::Request(partial) => {
            if pending.len() >= PENDING_LIMIT {
                evict_oldest_pending_visit(client, pending).await;
            }
            pending.insert(
                partial.visit_id.clone(),
                PendingVisit {
                    visit_id: partial.visit_id,
                    target: partial.target,
                    user_agent: partial.user_agent,
                    cdt: partial.cdt,
                    inserted_at: Utc::now(),
                },
            );
        }
        AIChatbotEvent::Response(response) => match pending.remove(&response.visit_id) {
            Some(entry) => {
                send_tracking_request(
                    client,
                    &entry,
                    response.http_status,
                    response.bw_bytes,
                    response.pf_srv,
                )
                .await;
            }
            None => {
                tracing::debug!("Received response for unknown visit {}", response.visit_id);
            }
        },
    }
}

/// When the pending map is full, drop the oldest visit so the map stays bounded.
async fn evict_oldest_pending_visit(
    client: &reqwest::Client,
    pending: &mut HashMap<AIChatbotVisitId, PendingVisit>,
) {
    let oldest = pending
        .iter()
        .min_by_key(|(_, entry)| entry.inserted_at)
        .map(|(k, _)| k.clone());
    match oldest.and_then(|key| pending.remove(&key)) {
        Some(entry) => send_tracking_request(client, &entry, 0, 0, 0).await,
        None => {}
    }
}

async fn send_tracking_request(
    client: &reqwest::Client,
    entry: &PendingVisit,
    http_status: u16,
    bw_bytes: u32,
    pf_srv: u64,
) {
    let name = entry.user_agent.as_str();
    let tracking_url = build_matomo_url(
        &entry.target,
        name,
        &entry.cdt,
        http_status,
        bw_bytes,
        pf_srv,
    );

    match client.get(&tracking_url).send().await {
        Ok(_r) => tracing::debug!("Submitted AI Chatbot ({}) tracking to Matomo.", name),
        Err(e) => {
            tracing::warn!(
                "Failed to send bot tracking event for {}:\n{}",
                entry.visit_id,
                e
            )
        }
    };
}

/// Send tracking requests for visits that exceeded `max_age` (or all of them on shutdown).
pub async fn flush_stale_visits(
    client: &reqwest::Client,
    pending: &mut HashMap<AIChatbotVisitId, PendingVisit>,
    max_age: Option<Duration>,
) {
    let now = Utc::now();
    let stale: Vec<AIChatbotVisitId> = pending
        .iter()
        .filter(|(_, entry)| match max_age {
            None => true,
            Some(max_age) => {
                (now - entry.inserted_at).num_milliseconds() > max_age.as_millis() as i64
            }
        })
        .map(|(k, _)| k.clone())
        .collect();

    for visit_id in &stale {
        match pending.remove(visit_id) {
            Some(entry) => send_tracking_request(client, &entry, 0, 0, 0).await,
            None => {}
        }
    }
}
