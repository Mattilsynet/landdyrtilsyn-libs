use serde::{Deserialize, Serialize};

/**
* Arkivsak benyttes på getSak og post sak i arkiv
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkivClientSak {
    #[serde(rename = "saksaar")]
    pub noarkaar: String,
    #[serde(rename = "sekvensnummer")]
    pub noarksaksnummer: String,

    #[serde(rename = "saksbehandlerId")]
    pub saksbehandler_id: Option<String>,

    pub ordningsverdi: String,

    pub tittel: String,

    pub skjermingshjemmel: Option<Kodeverk>,

    pub tilgangskode: Option<Kodeverk>,

    pub status: Option<Kodeverk>,

    pub lukket: bool,
    #[serde(rename = "enhetId")]
    pub enhet_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArkivClientJournalpost {
    /// Kortnavn til den som har fått journalposten tildelt i Elements
    pub fordelt_til: Option<String>,

    /// Id for journalpost
    pub journalpost_id: i32,

    /// Id for tilhørende hoveddokument
    pub hoveddokument_id: Option<String>,

    /// Journalposten sin tittel
    pub tittel: Option<String>,

    /// Filtypen til hoveddokumentet
    pub hoveddokument_filtype: Option<String>,

    /// Kode for dokumenttype journalen skal ha i Elements
    pub journalposttype: Kodeverk,

    /// Journalpoststatus benyttes for å angi hvor langt en journalpost er kommet
    pub journalstatus: Kodeverk,

    /// dokumentnummer på journalposten
    pub dokumentnummer: Option<i32>,

    /// Tittel på tilhørende hoveddokument
    pub dokument_tittel: Option<String>,

    /// Angir om journalposten har hoveddokument
    pub har_hoveddokument: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kodeverk {
    pub id: String,
    pub beskrivelse: String,
}
