use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TilsynsobjektKvittering {
    #[serde(rename = "tilsynsobjektId")]
    pub tilsynsobjekt_id: String,
    #[serde(rename = "tilsynskvitteringer")]
    pub tilsyns_kvitteringer: Vec<TidligereTilsynskvitteringInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TidligereTilsynskvitteringInfo {
    #[serde(rename = "externalTilsynsobjektId")]
    pub external_tilsynsobjekt_id: String,
    #[serde(rename = "noarksakAar")]
    pub noarksak_aar: String,
    #[serde(rename = "noarksakSekvensnummer")]
    pub noarksak_sekvensnummer: String,
    #[serde(rename = "feilmelding")]
    pub feilmelding: Option<String>,
    #[serde(rename = "feilmeldingKode")]
    pub feilmelding_kode: Option<i32>,
    pub status: Option<String>,
    pub tilsynsdato: Option<NaiveDate>,
    #[serde(rename = "tilsynskvitteringId")]
    pub tilsynskvittering_id: Option<i64>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "ansattNavn")]
    pub ansatt_navn: String,
    #[serde(rename = "antallBilder")]
    pub antall_bilder: i32,
    #[serde(rename = "antallKontrollpunkter")]
    pub antall_kontrollpunkter: i32,
    #[serde(rename = "unntattOffentlighet")]
    pub unntatt_offentlighet: bool,
}
