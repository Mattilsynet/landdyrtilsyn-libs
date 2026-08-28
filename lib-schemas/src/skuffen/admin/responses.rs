use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Snapshot-sammendrag utledet bare fra nåværende operasjonsrader.
///
/// Valideringsavvisning persisteres ikke lokalt og utledes derfor aldri her.
/// En kommando uten operasjoner er `uavklart`, aldri vacuous `fullfort`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminCommandUtfallV1 {
    Uavklart,
    KreverAvklaring,
    Fullfort,
    Feilet,
}

/// Kompakt entitet-identitet på en operasjonsrad.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminOperasjonEntitetV1 {
    pub skuffen_id: Uuid,
    pub entitet_type: String,
    pub client_reference: Option<Uuid>,
    pub arkiv_id: Option<String>,
}

/// Nåværende tilstand for én operasjon. Forsøkshistorikk inngår ikke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminOperasjonDetaljerV1 {
    pub operasjon_id: Uuid,
    pub operasjonstype: String,
    pub entitet: AdminOperasjonEntitetV1,
    pub sak_id: Uuid,
    pub status: String,
    pub attempt_no: i32,
    pub neste_forsok_at: Option<DateTime<Utc>>,
    pub blokkert_av: Option<Uuid>,
    pub siste_detalj: Option<String>,
    pub sendt_at: Option<DateTime<Utc>>,
    pub ferdig_at: Option<DateTime<Utc>>,
    pub varslet_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminCommandResponseV1 {
    pub command_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub command_type: String,
    pub mottatt_at: DateTime<Utc>,
    pub dispatchet_at: Option<DateTime<Utc>>,
    pub dekomponert_at: Option<DateTime<Utc>>,
    pub utfall: AdminCommandUtfallV1,
    pub operasjoner: Vec<AdminOperasjonDetaljerV1>,
}

/// Full entitet-identitet slik `entitet` lagrer den.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminEntitetIdentitetV1 {
    pub skuffen_id: Uuid,
    pub entitet_type: String,
    pub client_reference: Option<Uuid>,
    pub arkiv_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lagret korrespondansepart, flat og permissiv.
///
/// Speiler lagret JSON uten å konstruere command-side part-, id- eller
/// adressetyper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminKorrespondansepartV1 {
    pub rolle: String,
    pub navn: String,
    pub parttype: Option<String>,
    pub id_type: Option<String>,
    pub id: Option<String>,
    pub adresse: Option<String>,
    pub postnummer: Option<String>,
    pub poststed: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminDokumentV1 {
    pub identitet: AdminEntitetIdentitetV1,
    pub journalpost_id: Uuid,
    pub tilstand: String,
    pub rekkefolge: i32,
    pub er_hoveddokument: bool,
    pub tittel: Option<String>,
    pub filtype: Option<String>,
    pub dokument_referanse: Option<Uuid>,
    pub mal_referanse: Option<Uuid>,
    /// `None` for SQL `NULL`, `Some(vec![])` for lagret tom liste.
    pub felter: Option<Vec<String>>,
    pub rendered_dokument_referanse: Option<Uuid>,
    pub opprettet_av_command_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminJournalpostV1 {
    pub identitet: AdminEntitetIdentitetV1,
    pub sak_id: Uuid,
    pub tilstand: String,
    pub journalposttype: String,
    pub med_utsending: bool,
    pub tittel: Option<String>,
    pub dokument_dato: Option<String>,
    /// Journalpostens egen saksbehandler, som er et annet begrep enn sakens
    /// opprettelses-saksbehandler og enn saksansvarlig.
    pub saksbehandler_id: Option<String>,
    pub saksbehandler_enhet: Option<String>,
    pub tilgangskode: Option<String>,
    pub tilgangshjemmel: Option<String>,
    /// `None` for SQL `NULL`, `Some(vec![])` for lagret tom liste.
    pub korrespondanseparter: Option<Vec<AdminKorrespondansepartV1>>,
    pub kildesystem: Option<String>,
    pub opprettet_av_command_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dokumenter: Vec<AdminDokumentV1>,
}

/// Materialisert lokal sak-state.
///
/// `opprettelse_`-prefikset er bevisst: disse feltene er input til
/// `OpprettSak`, ikke den nåværende saksansvarlige.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminSakFaktaV1 {
    pub tilstand: String,
    pub sakstittel: Option<String>,
    pub arkivdel: Option<String>,
    pub ordningsverdi: Option<String>,
    pub opprettelse_saksbehandler_id: Option<String>,
    pub opprettelse_saksbehandler_enhet: Option<String>,
    pub tilgangskode: Option<String>,
    pub tilgangshjemmel: Option<String>,
    pub oensket_saksansvarlig_id: Option<String>,
    pub oensket_saksansvarlig_enhet: Option<String>,
    pub naavaerende_saksansvarlig_id: Option<String>,
    pub naavaerende_saksansvarlig_enhet: Option<String>,
    pub opprettet_av_command_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub journalposter: Vec<AdminJournalpostV1>,
}

/// Lett operasjonssammendrag. Detaljer hentes via command-oppslaget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminOperasjonSammendragV1 {
    pub operasjon_id: Uuid,
    pub command_id: Uuid,
    pub operasjonstype: String,
    pub entitet_id: Uuid,
    pub status: String,
}

/// `fakta` er `None` når Skuffen har mintet identitet, men ennå ikke
/// materialisert `sak_tilstand`. Det er reparasjonsinformasjon, ikke en
/// manglende sak.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminSakResponseV1 {
    pub identitet: AdminEntitetIdentitetV1,
    pub fakta: Option<AdminSakFaktaV1>,
    pub operasjoner: Vec<AdminOperasjonSammendragV1>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tidspunkt() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-08-27T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn identitet(skuffen_id: Uuid) -> AdminEntitetIdentitetV1 {
        AdminEntitetIdentitetV1 {
            skuffen_id,
            entitet_type: "sak".to_string(),
            client_reference: None,
            arkiv_id: None,
            created_at: tidspunkt(),
            updated_at: tidspunkt(),
        }
    }

    #[test]
    fn alle_utfall_bruker_snake_case() {
        for (utfall, forventet) in [
            (AdminCommandUtfallV1::Uavklart, "uavklart"),
            (AdminCommandUtfallV1::KreverAvklaring, "krever_avklaring"),
            (AdminCommandUtfallV1::Fullfort, "fullfort"),
            (AdminCommandUtfallV1::Feilet, "feilet"),
        ] {
            assert_eq!(serde_json::to_value(utfall).unwrap(), json!(forventet));
            assert_eq!(
                serde_json::from_value::<AdminCommandUtfallV1>(json!(forventet)).unwrap(),
                utfall
            );
        }
    }

    #[test]
    fn identity_only_sak_serialiseres_med_eksplisitt_null_fakta() {
        let response = AdminSakResponseV1 {
            identitet: identitet(Uuid::new_v4()),
            fakta: None,
            operasjoner: Vec::new(),
        };

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(value.get("fakta"), Some(&serde_json::Value::Null));
        assert_eq!(
            value.get("identitet").unwrap().get("arkiv_id"),
            Some(&serde_json::Value::Null)
        );
        assert_eq!(
            serde_json::from_value::<AdminSakResponseV1>(value).unwrap(),
            response
        );
    }

    #[test]
    fn null_og_tom_liste_bevares_separat() {
        let journalpost_id = Uuid::new_v4();
        let dokument = AdminDokumentV1 {
            identitet: identitet(Uuid::new_v4()),
            journalpost_id,
            tilstand: "klar".to_string(),
            rekkefolge: 0,
            er_hoveddokument: true,
            tittel: None,
            filtype: None,
            dokument_referanse: None,
            mal_referanse: None,
            felter: None,
            rendered_dokument_referanse: None,
            opprettet_av_command_id: Uuid::new_v4(),
            created_at: tidspunkt(),
            updated_at: tidspunkt(),
        };

        let uten = serde_json::to_value(&dokument).unwrap();
        assert_eq!(uten.get("felter"), Some(&serde_json::Value::Null));

        let tom = AdminDokumentV1 {
            felter: Some(Vec::new()),
            ..dokument.clone()
        };
        assert_eq!(
            serde_json::to_value(&tom).unwrap().get("felter"),
            Some(&json!([]))
        );

        assert_ne!(dokument.felter, tom.felter);
    }

    #[test]
    fn sak_roundtrip_holder_saksbehandlerkontekstene_adskilt() {
        let sak_id = Uuid::new_v4();
        let command_id = Uuid::new_v4();
        let journalpost_id = Uuid::new_v4();

        let response = AdminSakResponseV1 {
            identitet: identitet(sak_id),
            fakta: Some(AdminSakFaktaV1 {
                tilstand: "opprettet".to_string(),
                sakstittel: Some("Tilsynssak".to_string()),
                arkivdel: Some("tilsynsdivisjonene".to_string()),
                ordningsverdi: Some("123".to_string()),
                opprettelse_saksbehandler_id: Some("A".to_string()),
                opprettelse_saksbehandler_enhet: Some("A-enhet".to_string()),
                tilgangskode: None,
                tilgangshjemmel: None,
                oensket_saksansvarlig_id: Some("B".to_string()),
                oensket_saksansvarlig_enhet: Some("B-enhet".to_string()),
                naavaerende_saksansvarlig_id: Some("C".to_string()),
                naavaerende_saksansvarlig_enhet: Some("C-enhet".to_string()),
                opprettet_av_command_id: command_id,
                created_at: tidspunkt(),
                updated_at: tidspunkt(),
                journalposter: vec![AdminJournalpostV1 {
                    identitet: AdminEntitetIdentitetV1 {
                        entitet_type: "journalpost".to_string(),
                        ..identitet(journalpost_id)
                    },
                    sak_id,
                    tilstand: "opprettet".to_string(),
                    journalposttype: "X".to_string(),
                    med_utsending: false,
                    tittel: Some("Internt notat".to_string()),
                    dokument_dato: Some("2026-01-01".to_string()),
                    saksbehandler_id: Some("D".to_string()),
                    saksbehandler_enhet: Some("D-enhet".to_string()),
                    tilgangskode: None,
                    tilgangshjemmel: None,
                    korrespondanseparter: Some(vec![AdminKorrespondansepartV1 {
                        rolle: "utsendingsmottaker".to_string(),
                        navn: "Testmottaker".to_string(),
                        parttype: None,
                        id_type: Some("organisasjonsnummer".to_string()),
                        id: Some("999999999".to_string()),
                        adresse: Some("Testveien 1".to_string()),
                        postnummer: Some("0001".to_string()),
                        poststed: Some("Oslo".to_string()),
                    }]),
                    kildesystem: None,
                    opprettet_av_command_id: command_id,
                    created_at: tidspunkt(),
                    updated_at: tidspunkt(),
                    dokumenter: Vec::new(),
                }],
            }),
            operasjoner: vec![AdminOperasjonSammendragV1 {
                operasjon_id: Uuid::new_v4(),
                command_id,
                operasjonstype: "opprett_sak".to_string(),
                entitet_id: sak_id,
                status: "ok".to_string(),
            }],
        };

        let value = serde_json::to_value(&response).unwrap();
        let fakta = value.get("fakta").unwrap();
        assert_eq!(
            fakta.get("opprettelse_saksbehandler_id").unwrap(),
            &json!("A")
        );
        assert_eq!(fakta.get("oensket_saksansvarlig_id").unwrap(), &json!("B"));
        assert_eq!(
            fakta.get("naavaerende_saksansvarlig_id").unwrap(),
            &json!("C")
        );
        assert_eq!(
            fakta.get("journalposter").unwrap()[0]
                .get("saksbehandler_id")
                .unwrap(),
            &json!("D")
        );

        assert_eq!(
            serde_json::from_value::<AdminSakResponseV1>(value).unwrap(),
            response
        );
    }

    #[test]
    fn responsen_aksepterer_lagrede_verdier_uten_command_side_validering() {
        let value = json!({
            "identitet": {
                "skuffen_id": Uuid::new_v4(),
                "entitet_type": "sak",
                "client_reference": null,
                "arkiv_id": "",
                "created_at": "2026-08-27T10:00:00Z",
                "updated_at": "2026-08-27T10:00:00Z"
            },
            "fakta": {
                "tilstand": "ukjent_historisk_tilstand",
                "sakstittel": "",
                "arkivdel": "HOVEDKONTORET",
                "ordningsverdi": null,
                "opprettelse_saksbehandler_id": null,
                "opprettelse_saksbehandler_enhet": null,
                "tilgangskode": "",
                "tilgangshjemmel": "utgått hjemmel",
                "oensket_saksansvarlig_id": null,
                "oensket_saksansvarlig_enhet": null,
                "naavaerende_saksansvarlig_id": null,
                "naavaerende_saksansvarlig_enhet": null,
                "opprettet_av_command_id": Uuid::new_v4(),
                "created_at": "2026-08-27T10:00:00Z",
                "updated_at": "2026-08-27T10:00:00Z",
                "journalposter": []
            },
            "operasjoner": []
        });

        let parsed: AdminSakResponseV1 = serde_json::from_value(value).unwrap();
        let fakta = parsed.fakta.unwrap();
        assert_eq!(fakta.tilstand, "ukjent_historisk_tilstand");
        assert_eq!(fakta.tilgangskode.as_deref(), Some(""));
        assert_eq!(parsed.identitet.arkiv_id.as_deref(), Some(""));
    }

    #[test]
    fn command_respons_roundtripper_med_optional_felter() {
        let response = AdminCommandResponseV1 {
            command_id: Uuid::new_v4(),
            correlation_id: None,
            command_type: "opprett_sak".to_string(),
            mottatt_at: tidspunkt(),
            dispatchet_at: None,
            dekomponert_at: None,
            utfall: AdminCommandUtfallV1::Uavklart,
            operasjoner: vec![AdminOperasjonDetaljerV1 {
                operasjon_id: Uuid::new_v4(),
                operasjonstype: "opprett_sak".to_string(),
                entitet: AdminOperasjonEntitetV1 {
                    skuffen_id: Uuid::new_v4(),
                    entitet_type: "sak".to_string(),
                    client_reference: Some(Uuid::new_v4()),
                    arkiv_id: None,
                },
                sak_id: Uuid::new_v4(),
                status: "krever_avklaring".to_string(),
                attempt_no: 3,
                neste_forsok_at: None,
                blokkert_av: None,
                siste_detalj: Some("ukjent utfall etter recovery".to_string()),
                sendt_at: Some(tidspunkt()),
                ferdig_at: None,
                varslet_at: None,
                created_at: tidspunkt(),
                updated_at: tidspunkt(),
            }],
        };

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(value.get("correlation_id"), Some(&serde_json::Value::Null));
        assert_eq!(value.get("utfall").unwrap(), &json!("uavklart"));
        assert_eq!(
            value.get("mottatt_at").unwrap(),
            &json!("2026-08-27T10:00:00Z")
        );
        assert_eq!(
            serde_json::from_value::<AdminCommandResponseV1>(value).unwrap(),
            response
        );
    }
}
