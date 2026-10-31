use std::time::Duration;

use chrono::{NaiveDate, TimeZone, Utc};

use crate::astronomy::comet::OrbitalElements;
use crate::astronomy::time::julian_day;
use crate::error::{AppError, AppResult};
use crate::models::Comet;

const COMET_ELEMENTS_URL: &str = "https://www.minorplanetcenter.net/iau/MPCORB/CometEls.txt";
const USER_AGENT: &str = concat!("Vigie/", env!("CARGO_PKG_VERSION"));
/// Facteur MPC entre le « paramètre de pente » publié et K : K = 2.5 × G.
const SLOPE_TO_K: f64 = 2.5;

/// Accès HTTP brut au Minor Planet Center (éléments orbitaux des comètes).
pub struct MpcClient {
    http: reqwest::Client,
}

impl MpcClient {
    pub fn new() -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { http })
    }

    /// Télécharge `CometEls.txt` tel quel.
    pub async fn fetch_comet_elements(&self) -> AppResult<String> {
        let body = self.http.get(COMET_ELEMENTS_URL).send().await?.error_for_status()?.text().await?;
        if parse_comet_elements(&body).is_empty() {
            return Err(AppError::InvalidResponse("Minor Planet Center: no comet found in CometEls.txt".into()));
        }
        Ok(body)
    }
}

fn field(line: &str, start: usize, end: usize) -> Option<&str> {
    line.get(start..end.min(line.len())).map(str::trim)
}

fn number(line: &str, start: usize, end: usize) -> Option<f64> {
    field(line, start, end)?.parse().ok()
}

/// Une ligne au format « Export Format for Comet Orbits » du MPC (colonnes fixes).
fn parse_line(line: &str) -> Option<Comet> {
    let year: i32 = field(line, 14, 18)?.parse().ok()?;
    let month: u32 = field(line, 19, 21)?.parse().ok()?;
    let day = number(line, 22, 29)?;
    let date = NaiveDate::from_ymd_opt(year, month, 1)?.and_hms_opt(0, 0, 0)?;
    let perihelion_jde = julian_day(&Utc.from_utc_datetime(&date)) + day - 1.0;

    let name = field(line, 102, 158)?.to_string();
    if name.is_empty() {
        return None;
    }
    let designation = name.split(" (").next().unwrap_or(&name).to_string();
    Some(Comet {
        designation,
        name,
        elements: OrbitalElements {
            perihelion_jde,
            perihelion_distance_au: number(line, 30, 39)?,
            eccentricity: number(line, 41, 49)?,
            argument_of_perihelion_deg: number(line, 51, 59)?,
            ascending_node_deg: number(line, 61, 69)?,
            inclination_deg: number(line, 71, 79)?,
        },
        // Sans magnitude absolue, la luminosité est inconnue : comète ignorée.
        absolute_magnitude: number(line, 91, 95)?,
        slope_k: number(line, 96, 100)? * SLOPE_TO_K,
    })
}

/// Parse `CometEls.txt` ; les lignes incomplètes sont ignorées.
pub fn parse_comet_elements(text: &str) -> Vec<Comet> {
    text.lines().filter_map(parse_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
0002P         2027 02 10.2274  0.338598  0.847321  187.2875  334.0199   11.3480  20261030  14.4  4.0  2P/Encke                                                 MPEC 2026-UA3
    CK23A030  2024 09 27.8381  0.391302  1.000181  308.5867   21.6795  139.0985  20261030   6.5  3.2  C/2023 A3 (Tsuchinshan-ATLAS)                            MPEC 2026-O98
    CK99X99X  2024 09 27.8381  0.391302  1.000181  308.5867   21.6795  139.0985  20261030             C/1999 X99 (sans magnitude)
garbage line
";

    #[test]
    fn parses_fixed_columns() {
        let comets = parse_comet_elements(SAMPLE);
        assert_eq!(comets.len(), 2);
        let encke = &comets[0];
        assert_eq!(encke.designation, "2P/Encke");
        assert!((encke.elements.perihelion_jde - 2_461_446.727_4).abs() < 1e-6);
        assert_eq!(encke.elements.perihelion_distance_au, 0.338_598);
        assert_eq!(encke.elements.inclination_deg, 11.348);
        assert_eq!(encke.absolute_magnitude, 14.4);
        assert_eq!(encke.slope_k, 10.0);
        let atlas = &comets[1];
        assert_eq!(atlas.designation, "C/2023 A3");
        assert_eq!(atlas.name, "C/2023 A3 (Tsuchinshan-ATLAS)");
        assert!(atlas.elements.eccentricity > 1.0);
    }

    /// Vérifie le format du fichier publié par le MPC (réseau requis) :
    /// `cargo test -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn downloads_comet_elements() {
        let text = MpcClient::new().unwrap().fetch_comet_elements().await.unwrap();
        assert!(parse_comet_elements(&text).len() > 500);
    }
}
