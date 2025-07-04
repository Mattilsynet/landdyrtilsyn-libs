use crate::arkiv::{add_jens_suffix, remove_jens_suffix};
use core::fmt;
use lib_schemas::arkiv::{Landkode, SaksTittel, Saksaar, sak::NySak};
use lib_schemas::sak::Sak;
use lib_schemas::{Tilgangshjemmel, Tilgangskode};
use serde::{Deserialize, Deserializer, Serialize};

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

    pub status: Kodeverk,

    pub lukket: bool,
    #[serde(rename = "enhetId")]
    pub enhet_id: String,
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

impl From<Tilgangshjemmel> for Kodeverk {
    fn from(value: Tilgangshjemmel) -> Self {
        let hjemmel = value.hjemmel();
        Kodeverk {
            id: format!("TILGANGSHJEMMEL${hjemmel}"),
            beskrivelse: hjemmel.to_string(),
        }
    }
}

impl From<Tilgangskode> for Kodeverk {
    fn from(value: Tilgangskode) -> Self {
        match value {
            Tilgangskode::UnntattOffentlighet => Kodeverk {
                id: "TILGANGSKODE$UO".to_string(),
                beskrivelse: "".to_string(),
            },
        }
    }
}

/**
* Arkivsak benyttes på getSak og post sak i arkiv
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArkivSak {
    #[serde(rename(serialize = "saksaar"))]
    pub noarkaar: String,
    #[serde(rename(serialize = "sekvensnummer"))]
    pub noarksaksnummer: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dokument {
    /// Brukernavn til innspektøren som lager dokumentet
    #[serde(rename = "utarbeidetAvBrukernavn")]
    pub utarbeidet_av_brukernavn: String,

    /// Tittel på dokumentet
    #[serde(rename = "dokumentTittel")]
    pub dokument_tittel: String,

    /// Innholdet til dokumentet. Base64 Encodet
    #[serde(rename = "filinnhold")]
    pub filinnhold: String, // Base64 encoded content

    /// Filnavn for dokumentet
    #[serde(rename = "filnavn")]
    pub filnavn: String,
}

impl fmt::Display for Dokument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dokument: {{\n\tutarbeidet_av_brukernavn: {},\n\tdokument_tittel: {},\n\tfilinnhold: (Base64 encoded),\n\tfilnavn: {}\n}}",
            self.utarbeidet_av_brukernavn, self.dokument_tittel, self.filnavn
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArkivPostPdf {
    pub pdf: Vec<u8>,
    pub arkiv_sak: ArkivSak,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArkivPdfKvittering {
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_null_as_none"
    )]
    pub hoveddokument_id: Option<String>,

    pub journalpost_id: String,
    #[serde(rename(
        serialize = "noarksakSekvensnummer",
        deserialize = "noarksakSekvensnummer"
    ))]
    pub noarksaksnummer: String,
    #[serde(rename(serialize = "noarksakAar", deserialize = "noarksakAar"))]
    pub noarkaar: String,
}

// Custom deserialization to treat "null" as None
fn deserialize_null_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    if opt.as_deref() == Some("null") {
        Ok(None)
    } else {
        Ok(opt)
    }
}

// Define the ArkivSakArkivering struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkivSakArkivering {
    /// Hvilket år saken tilhører
    #[serde(rename = "saksaar")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noarkaar: Option<Saksaar>,

    /// Sekvensnummeret til saken
    #[serde(rename = "sekvensnummer")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noarksaksnummer: Option<String>,

    /// Id til saksbehandler
    //#[serde(rename(serialize = "saksbehandlerId"))]
    #[serde(rename = "saksbehandlerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saksbehandler_id: Option<String>,

    #[serde(rename = "enhetId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mt_enhet: Option<String>,

    /// Ordningsverdi til saken. Skal være på format kodetype$kode
    pub ordningsverdi: String,

    /// Saken sin tittel
    //#[serde(rename = "sakstittel")]
    pub tittel: SaksTittel,

    /// Hjemmel for at saken skal være unntatt fra offentigheten. Kode fra kodeverk TILGANGSHJEMMEL.
    /// Skal være på format kodetype$kode
    #[serde(
        rename = "tilgangshjemmel",
        alias = "skjermingshjemmel",
        skip_serializing_if = "Option::is_none"
    )]
    pub skjermingshjemmel: Option<Kodeverk>,

    /// Tilgangskode fra kodeverk TILGANGSKODE. Skal være på format kodetype$kode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilgangskode: Option<Kodeverk>,

    /// Status på saken
    #[serde(
        rename = "saksstatus",
        alias = "status",
        skip_serializing_if = "Option::is_none"
    )]
    pub status: Option<Kodeverk>,

    /// Status om at saken er lukket
    pub lukket: bool,

    /// VirksomhetsmappeId. Dersom denne er inkludert, vil den opprettede saken knyttes til virksomheten
    #[serde(rename = "virksomhetsmappeId", skip_serializing_if = "Option::is_none")]
    pub virksomhetsmappe_id: Option<String>,
}

impl From<NySak> for ArkivSakArkivering {
    fn from(value: NySak) -> Self {
        ArkivSakArkivering {
            noarkaar: None,
            noarksaksnummer: None,
            saksbehandler_id: value.saksbehandler_id,
            mt_enhet: value.mt_enhet.as_ref().map(|s| add_jens_suffix(s.clone())),
            ordningsverdi: value.ordningsverdi,
            tittel: value.tittel,
            skjermingshjemmel: value.skjermingshjemmel.map(|v| v.into()),
            tilgangskode: value.tilgangskode.map(|v| v.into()),
            status: None,
            lukket: false,
            virksomhetsmappe_id: None,
        }
    }
}

impl From<ArkivSakArkivering> for Sak {
    fn from(value: ArkivSakArkivering) -> Self {
        Sak {
            sekvensnummer: value.noarksaksnummer.unwrap(),
            saksaar: value.noarkaar.unwrap(),
            tittel: if value.tilgangskode.as_ref().map(|k| k.id.as_str()) == Some("UO") {
                value.tittel.uo_tittel()
            } else {
                value.tittel.clone()
            },
            enhet_id: remove_jens_suffix(value.mt_enhet.unwrap()),
            status: value
                .status
                .as_ref()
                .and_then(|kodeverk| kodeverk.id.split('$').next_back().map(str::to_string))
                .unwrap_or_else(|| panic!("Fant ikke statuskode i kodeverket {:?}", value.status)),
            saksbehandler_id: value
                .saksbehandler_id
                .as_ref()
                .map(|id| remove_jens_suffix(id.clone())),
            skjermingshjemmel: value
                .skjermingshjemmel
                .map(|kodeverk| kodeverk.id.split('$').next_back().unwrap().to_string()),
            tilgangskode: value
                .tilgangskode
                .map(|kodeverk| kodeverk.id.split('$').next_back().unwrap().to_string()),
            lukket: value.lukket,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AvsenderMottaker {
    mottaker: bool,
    navn: Option<String>,
    forsendelsesstatus: Option<String>,
    forsendelsesmaate: Option<String>,
    landkode: Option<Landkode>,
    #[serde(rename = "markerSomPerson")]
    marker_som_person: bool,
    #[serde(rename = "skjermesUOff")]
    skjermes_uoff: bool,
    #[serde(rename = "orgUnitAktoer")]
    org_unit_aktoer: bool,
    adresse: Option<String>,
    postnummer: Option<String>,
    poststed: Option<String>,
    kortnavn: Option<String>,
    brukernavn: Option<String>,
    epost: Option<String>,
}

pub struct AvsenderMottakerBuilder {
    mottaker: bool,
    navn: Option<String>,
    forsendelsesstatus: Option<String>,
    forsendelsesmaate: Option<String>,
    landkode: Option<Landkode>,
    marker_som_person: bool,
    skjermes_uoff: bool,
    org_unit_aktoer: bool,
    adresse: Option<String>,
    postnummer: Option<String>,
    poststed: Option<String>,
    kortnavn: Option<String>,
    brukernavn: Option<String>,
    epost: Option<String>,
}

impl Default for AvsenderMottakerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AvsenderMottakerBuilder {
    pub fn new() -> Self {
        AvsenderMottakerBuilder {
            mottaker: true,
            navn: None,
            forsendelsesstatus: None,
            forsendelsesmaate: None,
            landkode: None,
            marker_som_person: false,
            skjermes_uoff: false,
            org_unit_aktoer: true,
            adresse: None,
            postnummer: None,
            poststed: None,
            kortnavn: None,
            brukernavn: None,
            epost: None,
        }
    }

    pub fn mottaker(mut self, mottaker: bool) -> Self {
        self.mottaker = mottaker;
        self
    }

    pub fn navn(mut self, navn: String) -> Self {
        self.navn = Some(navn);
        self
    }

    pub fn forsendelsesstatus(mut self, forsendelsesstatus: String) -> Self {
        self.forsendelsesstatus = Some(forsendelsesstatus);
        self
    }

    pub fn forsendelsesmaate(mut self, forsendelsesmaate: String) -> Self {
        self.forsendelsesmaate = Some(forsendelsesmaate);
        self
    }

    pub fn landkode(mut self, landkode: Landkode) -> Self {
        self.landkode = Some(landkode);
        self
    }

    pub fn marker_som_person(mut self, marker_som_person: bool) -> Self {
        self.marker_som_person = marker_som_person;
        self
    }

    pub fn skjermes_uoff(mut self, skjermes_uoff: bool) -> Self {
        self.skjermes_uoff = skjermes_uoff;
        self
    }

    pub fn org_unit_aktoer(mut self, org_unit_aktoer: bool) -> Self {
        self.org_unit_aktoer = org_unit_aktoer;
        self
    }

    pub fn adresse(mut self, adresse: String) -> Self {
        self.adresse = Some(adresse);
        self
    }

    pub fn postnummer(mut self, postnummer: String) -> Self {
        self.postnummer = Some(postnummer);
        self
    }

    pub fn poststed(mut self, poststed: String) -> Self {
        self.poststed = Some(poststed);
        self
    }

    pub fn kortnavn(mut self, kortnavn: String) -> Self {
        self.kortnavn = Some(kortnavn);
        self
    }

    pub fn brukernavn(mut self, brukernavn: String) -> Self {
        self.brukernavn = Some(brukernavn);
        self
    }

    pub fn epost(mut self, epost: String) -> Self {
        self.epost = Some(epost);
        self
    }

    pub fn build(self) -> AvsenderMottaker {
        AvsenderMottaker {
            mottaker: self.mottaker,
            navn: self.navn,
            forsendelsesstatus: self.forsendelsesstatus,
            forsendelsesmaate: self.forsendelsesmaate,
            landkode: self.landkode,
            marker_som_person: self.marker_som_person,
            skjermes_uoff: self.skjermes_uoff,
            org_unit_aktoer: self.org_unit_aktoer,
            adresse: self.adresse,
            postnummer: self.postnummer,
            poststed: self.poststed,
            kortnavn: self.kortnavn,
            brukernavn: self.brukernavn,
            epost: self.epost,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArkiverDokument {
    #[serde(rename = "noarksakAar")]
    pub noarksak_aar: String,

    #[serde(rename = "noarksakSekvensnummer")]
    pub noarksak_sekvensnummer: String,

    #[serde(rename = "documentTitle")]
    pub dokument_tittel: String,

    #[serde(rename = "documentFilename")]
    pub dokument_filnavn: String,

    #[serde(rename = "journalpostTitle")]
    pub journalpost_title: String,

    #[serde(rename = "journalpostType")]
    pub journalpost_type: String,

    #[serde(rename = "saksbehandlerBrukernavn")]
    pub saksbehandler_brukernavn: String,

    #[serde(rename = "dokumentInnhold")]
    pub dokument_innhold: Vec<u8>,
    pub mottakere: Vec<AvsenderMottaker>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avsender: Option<AvsenderMottaker>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "unntattOffentlighetHjemmel")]
    pub unntatt_offentlighet_hjemmel: Option<String>,

    #[serde(rename = "enhetId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mt_enhet: Option<String>,
}

impl ArkiverDokument {
    // Constructor to set defaults where applicable
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        noarksak_aar: String,
        noarksak_sekvensnummer: String,
        dokument_tittel: String,
        journalpost_title: String,
        journalpost_type: String,
        saksbehandler_brukernavn: String,
        dokument_innhold: Vec<u8>,
        dokument_filnavn: String,
        mottakere: Vec<AvsenderMottaker>,
        avsender: Option<AvsenderMottaker>,
        unntatt_offentlighet_hjemmel: Option<String>,
        mt_enhet: Option<String>,
    ) -> Self {
        ArkiverDokument {
            noarksak_aar,
            noarksak_sekvensnummer,
            dokument_tittel,
            dokument_filnavn,
            journalpost_title,
            journalpost_type,
            saksbehandler_brukernavn,
            dokument_innhold,
            mottakere,
            avsender,
            unntatt_offentlighet_hjemmel,
            mt_enhet,
        }
    }
}

// Implement Display for Kodeverk
impl fmt::Display for Kodeverk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, beskrivelse: {} }}",
            self.id, self.beskrivelse
        )
    }
}

// Implement Display for ArkivSakArkivering
impl fmt::Display for ArkivSakArkivering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArkivSakArkivering {{ ")?;

        write!(
            f,
            "noarkaar: {}, ",
            self.noarkaar
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )?;

        write!(
            f,
            "noarksaksnummer: {}, ",
            self.noarksaksnummer.as_ref().unwrap_or(&"None".to_string())
        )?;

        write!(
            f,
            "saksbehandler_id: {}, ",
            self.saksbehandler_id.clone().unwrap_or_default()
        )?;
        write!(f, "ordningsverdi: {}, ", self.ordningsverdi)?;
        write!(f, "tittel: {}, ", self.tittel)?;

        write!(
            f,
            "skjermingshjemmel: {}, ",
            self.skjermingshjemmel
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )?;

        write!(
            f,
            "tilgangskode: {}, ",
            self.tilgangskode
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )?;

        write!(
            f,
            "status: {}, ",
            self.status
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )?;

        write!(f, "lukket: {}, ", self.lukket)?;

        write!(
            f,
            "virksomhetsmappe_id: {} ",
            self.virksomhetsmappe_id
                .as_ref()
                .unwrap_or(&"None".to_string())
        )?;

        write!(f, "}}")
    }
}

// Implement Display for AvsenderMottaker
impl fmt::Display for AvsenderMottaker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AvsenderMottaker {{ ")?;
        write!(f, "mottaker: {}, ", self.mottaker)?;
        write!(f, "navn: {}, ", self.navn.as_deref().unwrap_or("None"))?;
        write!(
            f,
            "forsendelsesstatus: {}, ",
            self.forsendelsesstatus.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "forsendelsesmaate: {}, ",
            self.forsendelsesmaate.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "landkode: {}, ",
            self.landkode
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "None".to_string())
        )?;
        write!(f, "marker_som_person: {}, ", self.marker_som_person)?;
        write!(f, "skjermes_uoff: {}, ", self.skjermes_uoff)?;
        write!(f, "org_unit_aktoer: {}, ", self.org_unit_aktoer)?;
        write!(
            f,
            "adresse: {}, ",
            self.adresse.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "postnummer: {}, ",
            self.postnummer.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "poststed: {}, ",
            self.poststed.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "kortnavn: {}, ",
            self.kortnavn.as_deref().unwrap_or("None")
        )?;
        write!(
            f,
            "brukernavn: {}, ",
            self.brukernavn.as_deref().unwrap_or("None")
        )?;
        write!(f, "epost: {} ", self.epost.as_deref().unwrap_or("None"))?;
        write!(f, "}}")
    }
}

// Implement Display for ArkiverDokument
impl fmt::Display for ArkiverDokument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ArkiverDokument {{")?;

        writeln!(f, "  noarksak_aar: {},", self.noarksak_aar)?;
        writeln!(
            f,
            "  noarksak_sekvensnummer: {},",
            self.noarksak_sekvensnummer
        )?;
        writeln!(f, "  dokument_tittel: {},", self.dokument_tittel)?;
        writeln!(f, "  dokument_filnavn: {},", self.dokument_filnavn)?;
        writeln!(f, "  journalpost_title: {},", self.journalpost_title)?;
        writeln!(f, "  journalpost_type: {},", self.journalpost_type)?;
        writeln!(
            f,
            "  saksbehandler_brukernavn: {},",
            self.saksbehandler_brukernavn
        )?;
        writeln!(
            f,
            "  dokument_innhold: [binary data of length {} bytes],",
            self.dokument_innhold.len()
        )?;

        // For mottakere, display each one using their Display implementation
        writeln!(f, "  mottakere: [")?;
        for mottaker in &self.mottakere {
            writeln!(f, "    {mottaker},")?;
        }
        writeln!(f, "  ],")?;

        // avsender is an Option
        writeln!(
            f,
            "  avsender: {},",
            match &self.avsender {
                Some(avsender) => avsender.to_string(),
                None => "None".to_string(),
            }
        )?;

        writeln!(
            f,
            "  unntatt_offentlighet_hjemmel: {}",
            self.unntatt_offentlighet_hjemmel
                .as_deref()
                .unwrap_or("None")
        )?;

        write!(f, "}}")
    }
}
