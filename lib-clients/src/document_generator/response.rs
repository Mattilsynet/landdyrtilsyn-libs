use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VedleggDokument {
    pub body: String,
    pub locale: String,
    #[serde(rename(serialize = "vaarRef"))]
    pub vaar_ref: String,
    #[serde(rename(serialize = "refPrefix"))]
    pub ref_prefix: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterntDokument {
    pub body: String,
    pub from: Avsender,
    #[serde(rename(serialize = "hjemmelForUnntattOffentlighet"))]
    pub hjemmel_for_unntatt_offentlighet: String,
    pub saksbehandler: Option<Saksbehandler>,
    pub title: String,
    pub to: String,
    #[serde(rename(serialize = "vaarRef"))]
    pub vaar_ref: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Saksbehandler {
    pub name: String,
    #[serde(rename(serialize = "phoneNumber"))]
    pub phone_number: Option<String>,
    pub region: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Avsender {
    #[serde(rename(serialize = "avsenderNavn"))]
    pub avsender_navn: Option<String>,
    #[serde(rename(serialize = "avsenderLinje1"))]
    pub avsender_linje1: Option<String>,
    #[serde(rename(serialize = "avsenderLinje2"))]
    pub avsender_linje2: Option<String>,
}

pub struct InterntDokumentBuilder {
    body: Option<String>,
    from: Option<Avsender>,
    hjemmel_for_unntatt_offentlighet: Option<String>,
    saksbehandler: Option<Saksbehandler>,
    title: Option<String>,
    to: Option<String>,
    vaar_ref: Option<String>,
}

impl Default for InterntDokumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InterntDokumentBuilder {
    pub fn new() -> Self {
        Self {
            body: None,
            from: None,
            hjemmel_for_unntatt_offentlighet: None,
            saksbehandler: None,
            title: None,
            to: None,
            vaar_ref: None,
        }
    }

    // Setter methods
    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn from(mut self, from: Avsender) -> Self {
        self.from = Some(from);
        self
    }

    pub fn hjemmel_for_unntatt_offentlighet(mut self, hjemmel: String) -> Self {
        self.hjemmel_for_unntatt_offentlighet = Some(hjemmel);
        self
    }

    pub fn saksbehandler(mut self, saksbehandler: Saksbehandler) -> Self {
        self.saksbehandler = Some(saksbehandler);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn to(mut self, to: String) -> Self {
        self.to = Some(to);
        self
    }

    pub fn vaar_ref(mut self, vaar_ref: String) -> Self {
        self.vaar_ref = Some(vaar_ref);
        self
    }

    pub fn build(self) -> Result<InterntDokument, &'static str> {
        Ok(InterntDokument {
            body: self.body.ok_or("Missing body")?,
            from: self.from.ok_or("Missing from")?,
            hjemmel_for_unntatt_offentlighet: self
                .hjemmel_for_unntatt_offentlighet
                .ok_or("Missing hjemmel_for_unntatt_offentlighet")?,
            saksbehandler: self.saksbehandler,
            title: self.title.ok_or("Missing title")?,
            to: self.to.ok_or("Missing to")?,
            vaar_ref: self.vaar_ref.ok_or("Missing vaar_ref")?,
        })
    }
}
