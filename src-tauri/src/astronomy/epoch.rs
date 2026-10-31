use super::angles::ARCSEC;
use super::coordinates::Equatorial;
#[cfg(test)]
use super::coordinates::{GeoSite, Horizontal, equatorial_to_horizontal};
use super::linalg::{Mat3, Vec3, add, normalize, scale};
use super::nutation::{Nutation, nutation};
use super::precession::precession_matrix;
use super::sidereal::{apparent_sidereal_time_deg, local_sidereal_time_deg};
use super::time::{J2000, terrestrial_time};
use super::vsop87::{self, Heliocentric};

/// Constante de l'aberration annuelle, en degrés.
const ABERRATION_CONSTANT_DEG: f64 = 20.495_52 * ARCSEC;

/// Tout ce qui ne dépend que de l'instant : échelles de temps, nutation, temps
/// sidéral, position de la Terre et matrices de changement de repère.
/// Calculé une fois par instant puis partagé par tous les objets.
#[derive(Debug, Clone, Copy)]
pub struct Epoch {
    /// Jour julien UT.
    pub jd: f64,
    /// Jour julien TT.
    pub jde: f64,
    pub nutation: Nutation,
    /// Temps sidéral apparent de Greenwich, en degrés.
    pub greenwich_sidereal_deg: f64,
    /// Position héliocentrique de la Terre (VSOP87D).
    pub earth: Heliocentric,
    /// Équateur moyen J2000 → équateur vrai de la date (précession puis nutation).
    j2000_to_true: Mat3,
    /// Écliptique moyen de la date → équateur vrai de la date.
    ecliptic_to_true: Mat3,
    /// Écliptique moyen de la date → équateur moyen J2000.
    ecliptic_to_j2000: Mat3,
    /// Aberration annuelle : κ × direction de la vitesse de la Terre (équateur vrai, rad).
    aberration: Vec3,
}

impl Epoch {
    pub fn from_julian_day(jd: f64) -> Self {
        Self::new(jd, terrestrial_time(jd))
    }

    /// Pour les exemples de Meeus, donnés en temps dynamique.
    #[cfg(test)]
    pub fn from_terrestrial_time(jde: f64) -> Self {
        Self::new(super::time::universal_time(jde), jde)
    }

    fn new(jd: f64, jde: f64) -> Self {
        let nutation = nutation(jde);
        let eps0 = nutation.mean_obliquity_deg;
        let eps = nutation.true_obliquity_deg();

        let precession = precession_matrix(J2000, jde);
        let nutation_matrix = Mat3::rotation_x(eps)
            .mul(&Mat3::rotation_z(nutation.longitude_deg))
            .mul(&Mat3::rotation_x(-eps0));
        let ecliptic_to_mean = Mat3::rotation_x(eps0);

        let earth = vsop87::earth(jde);
        // Vitesse orbitale de la Terre, supposée circulaire : perpendiculaire au
        // rayon vecteur, dans le sens des longitudes croissantes.
        let (sin_l, cos_l) = earth.longitude_rad.sin_cos();
        let velocity_direction = [-sin_l, cos_l, 0.0];
        let ecliptic_to_true = nutation_matrix.mul(&ecliptic_to_mean);

        Self {
            jd,
            jde,
            nutation,
            greenwich_sidereal_deg: apparent_sidereal_time_deg(jd, &nutation),
            earth,
            j2000_to_true: nutation_matrix.mul(&precession),
            ecliptic_to_true,
            ecliptic_to_j2000: precession.transpose().mul(&ecliptic_to_mean),
            aberration: scale(
                ecliptic_to_true.apply(velocity_direction),
                ABERRATION_CONSTANT_DEG.to_radians(),
            ),
        }
    }

    pub fn local_sidereal_deg(&self, longitude_east_deg: f64) -> f64 {
        local_sidereal_time_deg(self.greenwich_sidereal_deg, longitude_east_deg)
    }

    fn apparent(&self, true_equatorial: Vec3, with_aberration: bool) -> Equatorial {
        let mut u = normalize(true_equatorial);
        if with_aberration {
            u = add(u, self.aberration);
        }
        Equatorial::from_vector(u)
    }

    /// Direction apparente d'un vecteur de l'équateur moyen J2000 (étoiles,
    /// comètes) : précession, nutation et aberration annuelle.
    pub fn apparent_from_j2000(&self, v: Vec3) -> Equatorial {
        self.apparent(self.j2000_to_true.apply(v), true)
    }

    /// Direction apparente d'un vecteur de l'écliptique moyen de la date.
    /// `with_aberration` vaut `false` pour la Lune, dont la théorie l'inclut déjà.
    pub fn apparent_from_ecliptic(&self, v: Vec3, with_aberration: bool) -> Equatorial {
        self.apparent(self.ecliptic_to_true.apply(v), with_aberration)
    }

    /// Vecteur de l'écliptique moyen de la date exprimé dans l'équateur moyen J2000.
    pub fn ecliptic_to_j2000(&self, v: Vec3) -> Vec3 {
        self.ecliptic_to_j2000.apply(v)
    }

    /// Hauteur et azimut géométriques (sans réfraction) pour un lieu.
    #[cfg(test)]
    pub fn horizontal(&self, position: Equatorial, site: &GeoSite) -> Horizontal {
        equatorial_to_horizontal(position, self.local_sidereal_deg(site.longitude_deg), site.latitude_deg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::linalg::norm;

    #[test]
    fn aberration_is_about_twenty_arcseconds() {
        let epoch = Epoch::from_julian_day(2_461_344.5);
        assert!((norm(epoch.aberration).to_degrees() / ARCSEC - 20.4955).abs() < 1e-3);
    }

    #[test]
    fn time_scales_differ_by_delta_t() {
        let epoch = Epoch::from_julian_day(2_461_344.5);
        let delta_t_s = (epoch.jde - epoch.jd) * 86_400.0;
        assert!((60.0..80.0).contains(&delta_t_s));
    }
}
