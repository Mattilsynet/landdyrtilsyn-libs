/// Avdelingen et koststed hører til, lest ut av koststedskoden.
///
/// I tilsynsdivisjonene (M4) er avdelingen de fire første tegnene etterfulgt
/// av `00` (M42210 → M42200). På hovedkontoret (M3) er den de tre første
/// etterfulgt av `000` (M34600 → M34000). En avdeling er sin egen avdeling.
/// Divisjoner, hovedkontoret selv og andre koder har ingen avdeling.
pub fn avdelingskode(koststed: &str) -> Option<String> {
    let siffer = koststed.strip_prefix('M')?;
    if siffer.len() != 5 || !siffer.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (avdelingsprefiks, nuller) = match siffer.as_bytes() {
        [b'4', _, avdeling, ..] if *avdeling != b'0' => (&koststed[..4], "00"),
        [b'3', avdeling, ..] if *avdeling != b'0' => (&koststed[..3], "000"),
        _ => return None,
    };
    Some(format!("{avdelingsprefiks}{nuller}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seksjon_i_tilsynsdivisjon_hoerer_til_avdelingen_i_samme_hundre() {
        for (seksjon, avdeling) in [
            ("M42210", "M42200"),
            ("M42270", "M42200"),
            ("M42360", "M42300"),
            ("M43220", "M43200"),
            ("M43430", "M43400"),
        ] {
            assert_eq!(
                avdelingskode(seksjon).as_deref(),
                Some(avdeling),
                "{seksjon}"
            );
        }
    }

    #[test]
    fn avdeling_i_tilsynsdivisjon_er_sin_egen_avdeling() {
        for avdeling in ["M41100", "M42200", "M44500"] {
            assert_eq!(avdelingskode(avdeling).as_deref(), Some(avdeling));
        }
    }

    #[test]
    fn enhet_paa_hovedkontoret_hoerer_til_avdelingen_i_samme_tusen() {
        assert_eq!(avdelingskode("M34600").as_deref(), Some("M34000"));
        assert_eq!(avdelingskode("M34001").as_deref(), Some("M34000"));
        assert_eq!(avdelingskode("M34000").as_deref(), Some("M34000"));
    }

    #[test]
    fn divisjoner_hovedkontor_og_ukjente_koder_har_ingen_avdeling() {
        for kode in [
            "M42000", "M30000", "M00000", "M21100", "M4220", "M422100", "M42A10", "42210", "",
            "ukjent",
        ] {
            assert_eq!(avdelingskode(kode), None, "{kode}");
        }
    }
}
