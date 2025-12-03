use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tilgangshjemmel {
    SkjermingsHjemmelUoPublikum,
    SkjermingshjemmelUoMakks,
    SkjermingsHjemmelUoInternt,
}

impl Tilgangshjemmel {
    pub fn hjemmel(&self) -> &'static str {
        match self {
            Tilgangshjemmel::SkjermingsHjemmelUoPublikum => "Offl. § 24 andre ledd",
            Tilgangshjemmel::SkjermingshjemmelUoMakks => "Offl. § 24 andre ledd", //TODO skulle vært andre punktum, men finnes ikke i kodeverk
            Tilgangshjemmel::SkjermingsHjemmelUoInternt => "Offl. § 14 første ledd",
        }
    }
}
