//! Constellation contenant une position, d'après les limites officielles IAU
//! tabulées par Roman (1987, PASP 99, 695) dans l'équinoxe B1875.0.

use super::coordinates::Equatorial;
use super::precession::precess;

/// Équinoxe B1875.0, en jour julien TT.
pub const B1875_JDE: f64 = 2_405_889.258_55;

/// Une ligne de la table de Roman : bande de déclinaison inférieure et plage
/// d'ascension droite (heures), dans l'ordre de recherche.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryRow {
    pub ra_low_hours: f64,
    pub ra_high_hours: f64,
    pub dec_low_deg: f64,
    /// Indice de la constellation dans le catalogue appelant.
    pub constellation: usize,
}

/// Recherche dans la table (position en équinoxe B1875) : la première ligne
/// dont la déclinaison inférieure est sous l'objet et dont la plage d'AD le contient.
pub fn find_in_table(rows: &[BoundaryRow], b1875: Equatorial) -> Option<usize> {
    let ra_hours = b1875.ra_deg / 15.0;
    rows.iter()
        .find(|row| b1875.dec_deg >= row.dec_low_deg && ra_hours >= row.ra_low_hours && ra_hours < row.ra_high_hours)
        .map(|row| row.constellation)
}

/// Constellation d'une position rapportée à l'équinoxe `epoch_jde`.
pub fn constellation_of(rows: &[BoundaryRow], position: Equatorial, epoch_jde: f64) -> Option<usize> {
    find_in_table(rows, precess(position, epoch_jde, B1875_JDE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::time::J2000;

    /// Extrait de la table de Roman suffisant pour les cas testés.
    fn sample_rows() -> Vec<BoundaryRow> {
        let raw = [
            (0.0, 24.0, 88.0, 0),          // UMi
            (8.0, 14.5, 86.5, 0),          // UMi
            (21.0, 23.0, 86.1667, 0),      // UMi
            (18.0, 21.0, 86.0, 0),         // UMi
            (0.0, 8.0, 85.0, 1),           // Cep
            (18.175, 19.3583, 30.0, 2),    // Lyr
            (6.1167, 7.3667, -33.0, 3),    // CMa
            (11.8333, 12.8333, -64.0, 4),  // Cru
        ];
        raw.iter()
            .map(|&(ra_low_hours, ra_high_hours, dec_low_deg, constellation)| BoundaryRow {
                ra_low_hours,
                ra_high_hours,
                dec_low_deg,
                constellation,
            })
            .collect()
    }

    #[test]
    fn finds_constellations_of_bright_stars() {
        let rows = sample_rows();
        let cases = [
            ("Polaris", 37.954_561, 89.264_109, 0),
            ("Vega", 279.234_735, 38.783_689, 2),
            ("Sirius", 101.287_155, -16.716_116, 3),
            ("Acrux", 186.649_563, -63.099_093, 4),
        ];
        for (name, ra, dec, expected) in cases {
            let found = constellation_of(&rows, Equatorial { ra_deg: ra, dec_deg: dec }, J2000);
            assert_eq!(found, Some(expected), "{name}");
        }
    }
}
