use crate::error::ApiError;
use crate::error::Result as ErrorResult;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

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
    #[serde(rename = "tilgangshjemmel", skip_serializing_if = "Option::is_none")]
    pub skjermingshjemmel: Option<Kodeverk>,

    /// Tilgangskode fra kodeverk TILGANGSKODE. Skal være på format kodetype$kode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilgangskode: Option<Kodeverk>,

    /// Status på saken
    #[serde(rename = "saksstatus", skip_serializing_if = "Option::is_none")]
    pub status: Option<Kodeverk>,

    /// Status om at saken er lukket
    pub lukket: bool,

    /// VirksomhetsmappeId. Dersom denne er inkludert, vil den opprettede saken knyttes til virksomheten
    #[serde(rename = "virksomhetsmappeId", skip_serializing_if = "Option::is_none")]
    pub virksomhetsmappe_id: Option<String>,
}

impl ArkivSakArkivering {
    pub fn validate_skjerming(&self) -> ErrorResult<()> {
        let is_tittel_with_skjerming = self.tittel.0.contains('[');
        let has_both_skjerming_and_tilgangskode =
            self.tilgangskode.is_some() && self.skjermingshjemmel.is_some();

        if is_tittel_with_skjerming && !has_both_skjerming_and_tilgangskode {
            return Err(ApiError::ValidationError(
                "En skjermet arkivsak må ha tilgangskode og skjermingshjemmel.".to_string(),
            ));
        }

        Ok(())
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

#[derive(Debug, Clone)]
pub struct Landkode(String);

impl Serialize for Landkode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Landkode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Landkode(s))
    }
}

#[derive(Debug)]
pub enum LandkodeError {
    InvalidLength,
    InvalidCharacters,
}

impl FromStr for Landkode {
    type Err = LandkodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(LandkodeError::InvalidLength);
        }

        if !s.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(LandkodeError::InvalidCharacters);
        }

        Ok(Landkode(s.to_string()))
    }
}

impl std::fmt::Display for Landkode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Landkode {
    pub fn new(s: &str) -> Result<Self, LandkodeError> {
        s.parse()
    }
}

#[derive(Debug, Clone)]
pub struct Saksaar(pub String);

impl Serialize for Saksaar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Saksaar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Saksaar(s))
    }
}

impl fmt::Display for Saksaar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum SaksaarError {
    InvalidLength,
    InvalidCharacters,
}

impl FromStr for Saksaar {
    type Err = SaksaarError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(SaksaarError::InvalidLength);
        }

        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(SaksaarError::InvalidCharacters);
        }

        Ok(Saksaar(s.to_string()))
    }
}

impl Saksaar {
    pub fn new(s: &str) -> Result<Self, SaksaarError> {
        s.parse()
    }
}

/**
* SaksTittel benyttes på opprettelse av sak i arkiv
*/
const SAKSTITTEL_MAX_LENGTH: usize = 256;

#[derive(Debug, Clone)]
pub struct SaksTittel(String);

impl Serialize for SaksTittel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SaksTittel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SaksTittel(s))
    }
}

#[derive(Debug)]
pub enum SaksTittelError {
    Empty,
    TooLong,
}

impl FromStr for SaksTittel {
    type Err = SaksTittelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(SaksTittelError::Empty);
        }

        if trimmed.len() > SAKSTITTEL_MAX_LENGTH {
            return Err(SaksTittelError::TooLong);
        }

        Ok(SaksTittel(trimmed.to_string()))
    }
}

impl std::fmt::Display for SaksTittel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
            writeln!(f, "    {},", mottaker)?;
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
