use serde::{Deserialize, Serialize};

use crate::skuffen::{
    sak::{Ordningsverdi, Sakstittel},
    tilgang::Tilgang,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettSak {
    /// Sakstittel (required)
    /// Max length: 256, Min length: 1
    pub sakstittel: Sakstittel,

    /// Arkivdel som saken skal opprettes i. Eks: MATS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arkivdel: Option<String>,

    /// Journalenhet som saken skal opprettes i. Eks: DOKSENTER
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journalenhet: Option<String>,

    /// Saksbehandler
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksbehandler: Option<String>,

    /// SaksbehandlerEnhet
    #[serde(rename = "saksbehandlerEnhet", skip_serializing_if = "Option::is_none")]
    pub saksbehandler_enhet: Option<String>,

    /// Saksstatus
    /// Settes til B(Under behandling) ved opprett sak.

    /// Ordningsverdi slik den er registrert i Mattilsynets arkivnøkkel (required)
    /// Min length: 1
    pub ordningsverdi: Ordningsverdi,

    pub tilgang: Option<Tilgang>,

    /// VirksomhetsmappeId kommer fra saksbehandling i MATS.
    /// Dersom denne er inkludert, vil den opprettede saken knyttes til virksomheten via tilleggsattributt1 på saken.
    /// NB! Flere saker kan være knyttet mot samme VirksomhetsmappeId.
    #[serde(rename = "virksomhetsmappeId", skip_serializing_if = "Option::is_none")]
    pub virksomhetsmappe_id: Option<String>,
}
