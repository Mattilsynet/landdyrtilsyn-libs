//! # slack-error-notifier
//! En lettvekts‑crate som lar deg poste strukturerte feilmeldinger til en Slack‑kanal
//! via **Incoming Webhooks**. Craten har _ingen_ egne async‑runtimes, kun
//! **reqwest** for HTTP og **serde** for JSON, og kan derfor brukes i alt fra små
//! CLI‑verktøy til store mikrotjenester.
//!
//! ## Funksjoner
//! 1. **Plain‑text modus** (`send_error`) – raskt og kompatibelt med gamle Slack‑
//!    integrasjoner.
//! 2. **Attachment‑modus** (`send_error_with_attachment` / `slack_error!`) – gir
//!    fargekant, egne felter og generelt bedre lesbarhet.
//! 3. **Miljøbasert fargekoding** (valgfritt) – se eksempelet under.
//!
//! ## Rask start
//! ```rust,no_run
//! use lib_slack_notifier::{slack_error, SlackNotifier};
//! use std::env;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Hent webhook‑URL fra en sikker kilde
//! let url = env::var("SLACK_WEBHOOK_URL")?;
//! let notifier = SlackNotifier::new(url);
//!
//! // 2. Gjør noe som kan feile …
//! if let Err(e) = do_work().await {
//!     // 3. Post feilen til Slack. `slack_error!` samler fil + linje.
//!     slack_error!(notifier, e);
//! }
//! # Ok(())
//! # }
//!
//! async fn do_work() -> Result<(), Box<dyn std::error::Error>> {
//!     Err("Database connection failed".into())
//! }
//! ```
//!
//! ## Fargekoding etter miljø (valgfritt)
//! ```rust,ignore
//! let color = match std::env::var("RUST_ENV").as_deref() {
//!     Ok("production") => "danger",   // 🔴 rød
//!     Ok("staging")    => "warning",  // 🟡 gul
//!     _                => "good",     // 🟢 grønn (dev)
//! };
//! ```
//!
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
struct AttachmentPayload {
    text: String,
    attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Clone)]
struct Attachment {
    color: &'static str,
    fields: Vec<AttachmentField>,
}

#[derive(Debug, Serialize, Clone)]
struct AttachmentField {
    title: String,
    value: String,
    short: bool,
}

impl AttachmentPayload {
    fn new(app: &str, error: &str, location: &str) -> Self {
        Self {
            text: format!(":warning: Alle mann til slusene, *{app}* har en feil"),
            attachments: vec![Attachment {
                color: "danger",
                fields: vec![
                    AttachmentField {
                        title: "location".into(),
                        value: location.into(),
                        short: true,
                    },
                    AttachmentField {
                        title: "Error".into(),
                        value: error.into(),
                        short: false,
                    },
                ],
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlackNotifier {
    webhook_url: String,
    client: Client,
}

impl SlackNotifier {
    pub fn new<S: Into<String>>(webhook_url: S) -> Self {
        Self {
            webhook_url: webhook_url.into(),
            client: Client::new(),
        }
    }

    pub async fn send_message_with_attachment(
        &self,
        app: &str,
        error: &str,
        location: &str,
    ) -> Result<(), reqwest::Error> {
        let payload = AttachmentPayload::new(app, error, location);
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

/// Post en feilmelding _inkludert_ fil‑ og linjenummer.
///
/// Makroen evalueres til en `await`‑et `Result<(), reqwest::Error>` – akkurat som
/// [`SlackNotifier::send_error_with_attachment`]. Du velger dermed selv om du
/// vil bruke `?`, `unwrap()`, logge feilen eller ignorere den:
///
/// ```rust,no_run
/// # async fn demo() -> Result<(), reqwest::Error> {
/// use lib_slack_notifier::{slack_error, SlackNotifier};
/// let notifier = SlackNotifier::new("https://hooks…");
/// slack_error!(notifier, "Noe gikk galt"); // propagerer
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! slack_error {
    ($notifier:expr, $err:expr) => {{
        let location = format!("{}:{}", file!(), line!());
        $notifier
            .send_message_with_attachment(env!("CARGO_PKG_NAME"), &$err.to_string(), &location)
            .await
    }};
}
