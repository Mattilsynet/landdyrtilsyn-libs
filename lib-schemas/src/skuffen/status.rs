use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{journalpost::JournalpostId, sak::Saksnummer};

/// Kommandoens utfall, publisert på `arkiv.status.<command_id>.kommando`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkuffenKommandoStatusV1 {
    pub command_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    pub hendelse: SkuffenKommandoHendelse,
    /// `true` betyr at **utfallet er avgjort**, ikke at flere meldinger er
    /// utelukket. Operasjonsmeldinger kan fortsette å komme etterpå, fordi
    /// søskenoperasjoner kjører videre best effort.
    pub terminal: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<SkuffenStatusErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sak_client_reference: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksnummer: Option<Saksnummer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalpost_client_reference: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalpost_id: Option<JournalpostId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dokument_client_references: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Hendelser i en kommandos livsløp.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkuffenKommandoHendelse {
    /// Mottatt og lagt i kø.
    Mottatt,
    /// Validert og sendt til utførelse.
    Validert,
    /// Avvist ved validering. Terminal.
    Avvist,
    /// Dekomponert til operasjoner; utførelse pågår.
    Utfores,
    /// Alle operasjoner er terminalt ok. Terminal.
    Fullfort,
    /// Minst én operasjon feilet terminalt. Terminal.
    Feilet,
}

/// Én operasjons utfall, publisert på
/// `arkiv.status.<command_id>.operasjon.<operasjon_id>`.
///
/// En operasjon er ett arkivkall. Meldingene sendes ved forsøksutfall
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkuffenOperasjonStatusV1 {
    pub command_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    pub operasjon_id: Uuid,
    pub operasjonstype: SkuffenOperasjonstype,
    pub hendelse: SkuffenOperasjonHendelse,
    pub terminal: bool,
    /// Kun klientvennlig tekst.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<SkuffenStatusErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Hendelser i en operasjons livsløp.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkuffenOperasjonHendelse {
    /// Midlertidig feil. Nytt forsøk kommer.
    ForsokFeilet,
    /// Utført. Terminal.
    Ok,
    /// Kan ikke utføres. Terminal.
    Feilet,
    /// Utfallet er ukjent og må avklares manuelt. Ikke terminal.
    KreverAvklaring,
    /// Operasjonen har ikke fullført innen fristen. Advisory — forsøkene
    /// fortsetter, og ingenting avbrytes.
    Varsel,
}

/// Hvilket arkivkall operasjonen er.
///
/// `Journalfor`, `SettEkspedert` og `KlargjorForEkspedering` treffer samme
/// endepunkt i arkivet, men har ulik betydning og eksponeres derfor hver for
/// seg.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkuffenOperasjonstype {
    OpprettSak,
    RenderDokument,
    OpprettJournalpost,
    LeggTilVedlegg,
    Journalfor,
    SettEkspedert,
    KlargjorForEkspedering,
    AvventJournalfort,
    Avskriv,
    SettSaksansvarlig,
    AvsluttSak,
}

/// Client-safe error codes. Disse er bevisst coarse-grained.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkuffenStatusErrorCode {
    DuplicateRequest,
    InvalidRequest,
    NotFound,
    Conflict,
    PrerequisitePending,
    TemporaryUnavailable,
    ProcessingFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn uuid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn kommandostatus_serialiseres_med_kontekst() {
        let event = SkuffenKommandoStatusV1 {
            command_id: uuid(1),
            correlation_id: Some(uuid(2)),
            hendelse: SkuffenKommandoHendelse::Fullfort,
            terminal: true,
            message: "Forespørselen er fullført.".to_string(),
            error_code: None,
            sak_client_reference: Some(uuid(3)),
            saksnummer: Some(Saksnummer::new("2026/123").unwrap()),
            journalpost_client_reference: Some(uuid(4)),
            journalpost_id: Some(JournalpostId("jp-123".to_string())),
            dokument_client_references: Some(vec![uuid(5), uuid(6)]),
            timestamp: Some("2026-01-01T12:00:00Z".to_string()),
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(value["hendelse"], "fullfort");
        assert_eq!(value["terminal"], true);
        assert_eq!(value["saksnummer"], "2026/123");
        assert_eq!(value["journalpost_id"], "jp-123");
        assert_eq!(
            serde_json::from_value::<SkuffenKommandoStatusV1>(value).unwrap(),
            event
        );
    }

    #[test]
    fn kommandostatus_utelater_tomme_felter() {
        let event = SkuffenKommandoStatusV1 {
            command_id: uuid(7),
            correlation_id: None,
            hendelse: SkuffenKommandoHendelse::Mottatt,
            terminal: false,
            message: "Forespørselen er mottatt.".to_string(),
            error_code: None,
            sak_client_reference: None,
            saksnummer: None,
            journalpost_client_reference: None,
            journalpost_id: None,
            dokument_client_references: None,
            timestamp: None,
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(
            value,
            json!({
                "command_id": "00000000-0000-0000-0000-000000000007",
                "hendelse": "mottatt",
                "terminal": false,
                "message": "Forespørselen er mottatt."
            })
        );
    }

    #[test]
    fn operasjonstatus_serialiseres() {
        let event = SkuffenOperasjonStatusV1 {
            command_id: uuid(8),
            correlation_id: None,
            operasjon_id: uuid(9),
            operasjonstype: SkuffenOperasjonstype::KlargjorForEkspedering,
            hendelse: SkuffenOperasjonHendelse::ForsokFeilet,
            terminal: false,
            message: "Midlertidig feil. Nytt forsøk kommer.".to_string(),
            error_code: Some(SkuffenStatusErrorCode::TemporaryUnavailable),
            attempt: Some(3),
            timestamp: None,
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(value["operasjonstype"], "klargjor_for_ekspedering");
        assert_eq!(value["hendelse"], "forsok_feilet");
        assert_eq!(value["error_code"], "TEMPORARY_UNAVAILABLE");
        assert_eq!(value["attempt"], 3);
        assert_eq!(
            serde_json::from_value::<SkuffenOperasjonStatusV1>(value).unwrap(),
            event
        );
    }

    #[test]
    fn varsel_er_ikke_terminalt() {
        let value = json!({
            "command_id": "00000000-0000-0000-0000-00000000000a",
            "operasjon_id": "00000000-0000-0000-0000-00000000000b",
            "operasjonstype": "avvent_journalfort",
            "hendelse": "varsel",
            "terminal": false,
            "message": "Operasjonen har ikke fullført innen fristen."
        });

        let event: SkuffenOperasjonStatusV1 = serde_json::from_value(value).unwrap();

        assert_eq!(event.hendelse, SkuffenOperasjonHendelse::Varsel);
        assert!(!event.terminal);
    }

    #[test]
    fn interne_felter_avvises() {
        let value = json!({
            "command_id": "00000000-0000-0000-0000-00000000000c",
            "hendelse": "feilet",
            "terminal": true,
            "message": "Forespørselen kunne ikke fullføres.",
            "internal_state": "do-not-leak"
        });

        let error = serde_json::from_value::<SkuffenKommandoStatusV1>(value).unwrap_err();

        assert!(error.to_string().contains("unknown field `internal_state`"));
    }

    #[test]
    fn alle_operasjonstyper_har_stabile_koder() {
        let forventet = [
            (SkuffenOperasjonstype::OpprettSak, "opprett_sak"),
            (SkuffenOperasjonstype::RenderDokument, "render_dokument"),
            (
                SkuffenOperasjonstype::OpprettJournalpost,
                "opprett_journalpost",
            ),
            (SkuffenOperasjonstype::LeggTilVedlegg, "legg_til_vedlegg"),
            (SkuffenOperasjonstype::Journalfor, "journalfor"),
            (SkuffenOperasjonstype::SettEkspedert, "sett_ekspedert"),
            (
                SkuffenOperasjonstype::KlargjorForEkspedering,
                "klargjor_for_ekspedering",
            ),
            (
                SkuffenOperasjonstype::AvventJournalfort,
                "avvent_journalfort",
            ),
            (SkuffenOperasjonstype::Avskriv, "avskriv"),
            (
                SkuffenOperasjonstype::SettSaksansvarlig,
                "sett_saksansvarlig",
            ),
            (SkuffenOperasjonstype::AvsluttSak, "avslutt_sak"),
        ];

        for (variant, kode) in forventet {
            assert_eq!(serde_json::to_value(variant).unwrap(), kode);
        }
    }

    #[test]
    fn alle_kommandohendelser_har_stabile_koder() {
        let forventet = [
            (SkuffenKommandoHendelse::Mottatt, "mottatt"),
            (SkuffenKommandoHendelse::Validert, "validert"),
            (SkuffenKommandoHendelse::Avvist, "avvist"),
            (SkuffenKommandoHendelse::Utfores, "utfores"),
            (SkuffenKommandoHendelse::Fullfort, "fullfort"),
            (SkuffenKommandoHendelse::Feilet, "feilet"),
        ];

        for (variant, kode) in forventet {
            assert_eq!(serde_json::to_value(variant).unwrap(), kode);
        }
    }
}
