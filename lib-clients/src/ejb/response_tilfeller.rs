use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const ALLOWED_FIELDS_TILFELLER: &[&str] = &[
    "idstring",
    "version",
    "typeid",
    "tilsynsobjektref",
    "virksomhetref",
    "registrertdato",
    "diagnoseid",
    "mistenktsykdomid",
    "diagnosegrunnlagid",
    "mistenktdato",
    "avkreftadato",
    "stadfestadato",
    "avsluttadato",
    "dtype",
    "meldtvirksomhetsnavn",
    "meldttilsynsobjekt",
    "sykdomstilfellemapperef",
    "artkategoriid",
    "samlebehandlingref",
    "innmeldernavn",
    "innmeldertlf",
    "doedevedmistenktdato",
    "sykevedmistenktdato",
    "totaltvedutbruddsdato",
    "doedevedavsluttetdato",
    "antallpaalagtslaktet",
    "antallslaktettilkonsum",
    "gaardsnummer",
    "bruksnummer",
    "merdnummer",
    "idpaafisk",
    "kommunenummer",
    "mistankegrunnlagid",
    "gbridentitet_idstring",
    "kontaktperson",
    "tlfnrkontaktperson",
    "hendelsesdato",
    "hendelsestidspunkt",
    "ugyldig",
    "gbrnummerref",
    "hendelsetype",
    "spesifiserthendelsetype",
    "beskrivelse",
    "detaljer",
    "strakstiltak",
    "tiltaksplan",
    "mottakeligevedmistanke",
    "sykevedstadfestelse",
    "doedevedstadfestelse",
    "utfoertavlivetdestruert",
    "antallvaksinert",
    "paalagtavlivetslaktet",
    "utfoertavlivetslaktet",
    "produsentnummer",
    "produsentref",
    "paavistvilis",
    "paavistintermedia",
    "paavistglabrata",
    "paavistpilosissima",
    "antallplanterpaavist",
    "sistpaavist",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponseTilfelle {
    pub results: Vec<Sykdomstilfelle>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sykdomstilfelle {
    pub idstring: String,
    pub version: i32,
    pub typeid: String,
    pub tilsynsobjektref: String,
    pub virksomhetref: String,

    #[serde(with = "date_format")]
    pub registrertdato: Option<DateTime<Utc>>,

    pub diagnoseid: Option<String>,
    pub mistenktsykdomid: Option<String>,
    pub diagnosegrunnlagid: Option<String>,

    #[serde(with = "date_format")]
    pub mistenktdato: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub avkreftadato: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub stadfestadato: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub avsluttadato: Option<DateTime<Utc>>,

    pub dtype: String,
    pub meldtvirksomhetsnavn: Option<String>,
    pub meldttilsynsobjekt: Option<String>,
    pub sykdomstilfellemapperef: Option<String>,
    pub artkategoriid: Option<String>,
    pub samlebehandlingref: Option<String>,
    pub innmeldernavn: Option<String>,
    pub innmeldertlf: Option<String>,

    pub doedevedmistenktdato: Option<i32>,
    pub sykevedmistenktdato: Option<i32>,
    pub totaltvedutbruddsdato: Option<i32>,
    pub doedevedavsluttetdato: Option<i32>,
    pub antallpaalagtslaktet: Option<i32>,
    pub antallslaktettilkonsum: Option<i32>,

    pub gaardsnummer: Option<String>,
    pub bruksnummer: Option<String>,
    pub merdnummer: Option<String>,
    pub idpaafisk: Option<String>,
    pub kommunenummer: Option<String>,
    pub mistankegrunnlagid: Option<String>,
    pub gbridentitet_idstring: Option<String>,
    pub kontaktperson: Option<String>,
    pub tlfnrkontaktperson: Option<String>,

    #[serde(with = "date_format")]
    pub hendelsesdato: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    pub hendelsestidspunkt: Option<DateTime<Utc>>,

    pub ugyldig: i32,
    pub gbrnummerref: Option<String>,
    pub hendelsetype: Option<String>,
    pub spesifiserthendelsetype: Option<String>,
    pub beskrivelse: Option<String>,
    pub detaljer: Option<String>,
    pub strakstiltak: Option<String>,
    pub tiltaksplan: Option<String>,
    pub mottakeligevedmistanke: i32,
    pub sykevedstadfestelse: i32,
    pub doedevedstadfestelse: i32,
    pub utfoertavlivetdestruert: i32,
    pub antallvaksinert: i32,
    pub paalagtavlivetslaktet: i32,
    pub utfoertavlivetslaktet: i32,
    pub produsentnummer: Option<String>,
    pub produsentref: Option<String>,
    pub paavistvilis: Option<String>,
    pub paavistintermedia: Option<String>,
    pub paavistglabrata: Option<String>,
    pub paavistpilosissima: Option<String>,
    pub antallplanterpaavist: i32,
    #[serde(with = "date_format")]
    pub sistpaavist: Option<DateTime<Utc>>,
}

pub mod date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT_DATETIME: &str = "%Y-%m-%d %H:%M:%S";
    const FORMAT_DATE: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&date.format(FORMAT_DATETIME).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;

        match s {
            Some(s) => {
                // Prøv først datetime format
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&s, FORMAT_DATETIME) {
                    return Ok(Some(Utc.from_utc_datetime(&dt)));
                }

                // Hvis det feiler, prøv dato format
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&s, FORMAT_DATE) {
                    if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                        return Ok(Some(Utc.from_utc_datetime(&dt)));
                    }
                }

                Err(serde::de::Error::custom(format!(
                    "Ugyldig datoformat. Forventet '{FORMAT_DATETIME}' eller '{FORMAT_DATE}'"
                )))
            }
            None => Ok(None),
        }
    }
}
