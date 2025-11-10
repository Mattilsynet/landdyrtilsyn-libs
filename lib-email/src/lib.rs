use crate::types::{Body, Email, EmailAddress, Message, Recipient};
use lib_nats::Client;

pub mod types;

/// Sender en epost med ferdig HTML template.
///
/// - ```email_subject``` er subjekt-feltet på eposten.
/// - ```content_header``` er tittelen inne i eposten.
/// - ```content_body``` er hovedteksten inne i eposten.
pub async fn send_formatted_email(
    nats_client: Client,
    email_address: String,
    email_subject: String,
    content_header: String,
    content_body: String,
) {
    let email_body = compose_html(content_header, content_body);
    let message = Message {
        subject: email_subject,
        to_recipients: vec![Recipient {
            email_address: Some(EmailAddress {
                address: "andreas.sigstad.lande@mattilsynet.no".to_string(),
            }),
        }],
        body: Some(Body {
            content_type: "HTML".to_string(),
            content: email_body,
        }),
        cc_recipients: Vec::new(),
        attachments: Vec::new(),
        internet_message_headers: Vec::new(),
        flag: None,
        save_to_sent_items: false,
    };

    let email = Email {
        message: Some(message),
    };

    send_email(email, nats_client).await;
}

#[tracing::instrument(name = "Sender epost til ansatt", skip(nats_client, email))]
async fn send_email(email: Email, nats_client: Client) {
    let nats_subject = "map-mailer".to_string();
    let payload = serde_json::to_vec(&email).expect("Failed to serialize email to JSON");
    match nats_client.request(nats_subject, payload.into()).await {
        Ok(_response) => {}
        Err(_e) => {
            tracing::error!("Failed to send email via NATS");

            // Send melding på slack
            // TODO
        }
    }
}

const EMAIL_TEMPLATE: &str = include_str!("email_template.html");

fn compose_html(header: String, body: String) -> String {
    let html_body = EMAIL_TEMPLATE
        .replace("{{HEADER}}", &header)
        .replace("{{BODY}}", &body);

    html_body.to_string()
}
