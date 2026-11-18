use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    sak::{Ordningsverdi, Sakstittel},
    tilgang::Tilgjengelighet,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OpprettSak {
    pub client_reference: Uuid,
    pub sakstittel: Sakstittel,
    pub arkivdel: Arkivdel,
    /// Skuffen kontrollerer ved mottak at minst ett felt er satt og at verdiene ikke er blanke.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksbehandler_id: Option<String>,
    /// Uten eksplisitt enhet fyller arkivet den ut fra saksbehandleren.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksbehandler_enhet: Option<String>,
    pub ordningsverdi: Ordningsverdi,
    pub tilgjengelighet: Tilgjengelighet,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kildesystem: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AvsluttSak {
    pub sak_key: crate::skuffen::query::queries::SakKey,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Arkivdel {
    Tilsynsdivisjonene,
    Hovedkontoret,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SettSaksansvarlig {
    pub sak_key: crate::skuffen::query::queries::SakKey,
    pub saksbehandler_id: String,
    pub saksbehandler_enhet: String,
}
