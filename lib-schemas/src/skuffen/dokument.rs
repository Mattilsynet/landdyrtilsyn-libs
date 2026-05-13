use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifier for dokumenter lagret i arkivet.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DokumentId(pub String);

impl DokumentId {
    /// Returner raw dokument id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Document metadata brukt i journalposter.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    /// Client side correlation for idempotency.
    pub client_reference: Uuid,
    /// Tittel for dokumentet.
    pub tittel: String,
    /// Dokumentets innholdsform.
    pub form: Dokumentform,
}

/// Dokumentets innholdsform.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Dokumentform {
    /// Ferdig opplastede dokumentbytes.
    Bytes {
        /// Archive reference til det lagrede dokumentet.
        dokument_referanse: Uuid,
        /// File type/extension (e.g. "pdf").
        filtype: String,
    },
    /// HTML-mal som rendres av Skuffen etter at deklarerte felter finnes.
    HtmlTemplate {
        /// Reference til opplastet HTML-mal.
        mal_referanse: Uuid,
        /// Felter malen trenger før rendering.
        felter: Vec<Felt>,
    },
}

/// Felt som kan substitueres inn i HTML-maler.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy, Hash)]
pub enum Felt {
    /// Saksnummer for saken dokumentet hører til.
    Saksnummer,
}

#[cfg(test)]
mod tests {
    use super::{Dokument, Dokumentform, Felt};
    use uuid::Uuid;

    #[test]
    fn dokument_bytes_roundtripper_med_default_serde_shape() {
        let dokument_referanse = Uuid::new_v4();
        let dokument = Dokument {
            client_reference: Uuid::new_v4(),
            tittel: "Rapport".to_string(),
            form: Dokumentform::Bytes {
                dokument_referanse,
                filtype: "PDF".to_string(),
            },
        };

        let json = serde_json::to_value(&dokument).expect("serialize dokument");

        assert_eq!(json["form"]["Bytes"]["dokument_referanse"], dokument_referanse.to_string());
        assert_eq!(json["form"]["Bytes"]["filtype"], "PDF");

        let roundtrip: Dokument = serde_json::from_value(json).expect("deserialize dokument");
        assert_eq!(roundtrip, dokument);
    }

    #[test]
    fn dokument_html_template_roundtripper_med_default_serde_shape() {
        let mal_referanse = Uuid::new_v4();
        let dokument = Dokument {
            client_reference: Uuid::new_v4(),
            tittel: "Vedtak".to_string(),
            form: Dokumentform::HtmlTemplate {
                mal_referanse,
                felter: vec![Felt::Saksnummer],
            },
        };

        let json = serde_json::to_value(&dokument).expect("serialize dokument");

        assert_eq!(json["form"]["HtmlTemplate"]["mal_referanse"], mal_referanse.to_string());
        assert_eq!(json["form"]["HtmlTemplate"]["felter"][0], "Saksnummer");

        let roundtrip: Dokument = serde_json::from_value(json).expect("deserialize dokument");
        assert_eq!(roundtrip, dokument);
    }

    #[test]
    fn ukjent_dokumentform_variant_avvises() {
        let json = serde_json::json!({
            "client_reference": Uuid::new_v4(),
            "tittel": "Rapport",
            "form": { "MarkdownTemplate": { "mal_referanse": Uuid::new_v4(), "felter": [] } }
        });

        let err = serde_json::from_value::<Dokument>(json).expect_err("unknown form variant");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn ukjent_felt_variant_avvises() {
        let json = serde_json::json!({
            "client_reference": Uuid::new_v4(),
            "tittel": "Vedtak",
            "form": { "HtmlTemplate": { "mal_referanse": Uuid::new_v4(), "felter": ["Ukjent"] } }
        });

        let err = serde_json::from_value::<Dokument>(json).expect_err("unknown felt variant");
        assert!(err.to_string().contains("unknown variant"));
    }
}
