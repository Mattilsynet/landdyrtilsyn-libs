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
    /// Skuffen krever både saksbehandler og enhet; tomme verdier avvises.
    pub saksbehandler_id: String,
    pub saksbehandler_enhet: String,
    pub ordningsverdi: Ordningsverdi,
    pub tilgjengelighet: Tilgjengelighet,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opprett_sak_avviser_ukjente_felt() {
        let gyldig = serde_json::json!({
            "client_reference": "00000000-0000-0000-0000-000000000000",
            "sakstittel": "Test",
            "arkivdel": "Tilsynsdivisjonene",
            "saksbehandler_id": "Z1",
            "saksbehandler_enhet": "42",
            "ordningsverdi": "123",
            "tilgjengelighet": "Offentlig"
        });
        assert!(serde_json::from_value::<OpprettSak>(gyldig.clone()).is_ok());

        let mut med_ukjent = gyldig;
        med_ukjent["evil_injected_field"] = serde_json::json!("x");
        assert!(serde_json::from_value::<OpprettSak>(med_ukjent).is_err());
    }
}
