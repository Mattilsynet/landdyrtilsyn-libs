use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    sak::{Ordningsverdi, Sakstittel},
    tilgang::Tilgang,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettSak {
    pub client_reference: Uuid,
    /// Sakstittel (required)
    /// Max length: 256, Min length: 1
    pub sakstittel: Sakstittel,

    /// Arkivdel som saken skal opprettes i.
    pub arkivdel: Arkivdel,

    /// Journalenhet som saken skal opprettes i. Settes alltid til DOKSENTER
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub journalenhet: Option<String>,

    /// Arkivet støtter å opprette en sak med bare enhet, eller uten enhet og uten saksbehandler.
    /// Men dette fører noen ganger til feil. Skuffen støtter ikke dette intil videre for å
    /// opprettholde kontroll over flyten med journalføring av avskriving.
    pub saksbehandler_id: String,
    pub saksbehandler_enhet: String,

    /// Saksstatus
    /// Settes til B(Under behandling) ved opprett sak.

    /// Ordningsverdi slik den er registrert i Mattilsynets arkivnøkkel (required)
    /// Min length: 1
    pub ordningsverdi: Ordningsverdi,

    /// Brukes ved skjerming.
    pub tilgang: Option<Tilgang>,
    // VirksomhetsmappeId kommer fra saksbehandling i MATS.
    // Dersom denne er inkludert, vil den opprettede saken knyttes til virksomheten via tilleggsattributt1 på saken.
    // Flere saker kan være knyttet mot samme VirksomhetsmappeId.
    // #[serde(rename = "virksomhetsmappeId", skip_serializing_if = "Option::is_none")]
    // pub virksomhetsmappe_id: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AvsluttSak {
    pub sak_key: crate::skuffen::query::queries::SakKey,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Arkivdel {
    Tilsynsdivisjonene, //Mappes til "SAK"
    Hovedkontoret,      //Mappes til "SAKHK"
}
