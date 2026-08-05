use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::skuffen::{
    sak::{Ordningsverdi, Sakstittel},
    tilgang::Tilgjengelighet,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct OpprettSak {
    pub client_reference: Uuid,
    pub sakstittel: Sakstittel,
    pub arkivdel: Arkivdel,
    /// Skuffen krever både saksbehandler og enhet; tomme verdier avvises.
    pub saksbehandler_id: String,
    pub saksbehandler_enhet: String,
    pub ordningsverdi: Ordningsverdi,
    pub tilgjengelighet: Tilgjengelighet,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AvsluttSak {
    pub sak_key: crate::skuffen::query::queries::SakKey,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub enum Arkivdel {
    Tilsynsdivisjonene,
    Hovedkontoret,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct SettSaksansvarlig {
    pub sak_key: crate::skuffen::query::queries::SakKey,
    pub saksbehandler_id: String,
    pub saksbehandler_enhet: String,
}
