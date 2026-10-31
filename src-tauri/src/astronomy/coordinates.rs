use super::angles::normalize_degrees;
use super::linalg::Vec3;

/// Coordonnées équatoriales (ascension droite, déclinaison), en degrés.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Equatorial {
    pub ra_deg: f64,
    pub dec_deg: f64,
}

/// Coordonnées écliptiques (longitude, latitude), en degrés.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ecliptic {
    pub lon_deg: f64,
    pub lat_deg: f64,
}

/// Coordonnées horizontales : azimut compté depuis le nord vers l'est.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Horizontal {
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
}

/// Lieu d'observation (latitude géodésique, longitude positive vers l'est).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoSite {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_m: f64,
}

/// Vecteur unitaire correspondant à des coordonnées sphériques (degrés).
pub fn spherical_to_vector(lon_deg: f64, lat_deg: f64) -> Vec3 {
    let (sin_lon, cos_lon) = lon_deg.to_radians().sin_cos();
    let (sin_lat, cos_lat) = lat_deg.to_radians().sin_cos();
    [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat]
}

/// Coordonnées sphériques (longitude dans [0, 360[, latitude) d'un vecteur quelconque.
pub fn vector_to_spherical(v: Vec3) -> (f64, f64) {
    let lon = normalize_degrees(v[1].atan2(v[0]).to_degrees());
    let lat = v[2].atan2(v[0].hypot(v[1])).to_degrees();
    (lon, lat)
}

impl Equatorial {
    pub fn to_vector(self) -> Vec3 {
        spherical_to_vector(self.ra_deg, self.dec_deg)
    }

    pub fn from_vector(v: Vec3) -> Self {
        let (ra_deg, dec_deg) = vector_to_spherical(v);
        Self { ra_deg, dec_deg }
    }
}

impl Ecliptic {
    pub fn to_vector(self) -> Vec3 {
        spherical_to_vector(self.lon_deg, self.lat_deg)
    }
}

/// Équatoriales → horizontales (Meeus, 13.5 et 13.6), azimut depuis le nord.
pub fn equatorial_to_horizontal(position: Equatorial, local_sidereal_deg: f64, latitude_deg: f64) -> Horizontal {
    let (sin_h, cos_h) = (local_sidereal_deg - position.ra_deg).to_radians().sin_cos();
    let (sin_d, cos_d) = position.dec_deg.to_radians().sin_cos();
    let (sin_p, cos_p) = latitude_deg.to_radians().sin_cos();
    let azimuth = (-cos_d * sin_h).atan2(sin_d * cos_p - cos_d * cos_h * sin_p);
    let altitude = (sin_p * sin_d + cos_p * cos_d * cos_h).clamp(-1.0, 1.0).asin();
    Horizontal {
        azimuth_deg: normalize_degrees(azimuth.to_degrees()),
        altitude_deg: altitude.to_degrees(),
    }
}

/// Distance angulaire entre deux directions, en degrés.
pub fn angular_separation_deg(a: Equatorial, b: Equatorial) -> f64 {
    let (va, vb) = (a.to_vector(), b.to_vector());
    let cross = [
        va[1] * vb[2] - va[2] * vb[1],
        va[2] * vb[0] - va[0] * vb[2],
        va[0] * vb[1] - va[1] * vb[0],
    ];
    let sin = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let cos = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    sin.atan2(cos).to_degrees()
}

/// Réfraction atmosphérique standard (10 °C, 1010 hPa) à ajouter à une
/// hauteur vraie, en degrés (Sæmundsson, Meeus 16.4). Sous −1°, l'effet est
/// ramené progressivement à zéro pour rester continu.
pub fn refraction_deg(true_altitude_deg: f64) -> f64 {
    let formula = |h: f64| 1.02 / (h + 10.3 / (h + 5.11)).to_radians().tan() / 60.0;
    match true_altitude_deg {
        h if h >= 90.0 => 0.0,
        h if h >= -1.0 => formula(h).max(0.0),
        h if h > -3.0 => formula(-1.0) * (h + 3.0) / 2.0,
        _ => 0.0,
    }
}

/// Rayon équatorial terrestre (Meeus, ch. 11), en mètres.
const EARTH_EQUATORIAL_RADIUS_M: f64 = 6_378_140.0;
/// Rapport b/a de l'ellipsoïde terrestre.
const EARTH_FLATTENING_RATIO: f64 = 0.996_647_19;
/// Parallaxe horizontale équatoriale du Soleil à 1 UA, en degrés (8.794″).
const SOLAR_PARALLAX_DEG: f64 = 8.794 / 3600.0;

/// Corrige une position géocentrique de la parallaxe pour un observateur au
/// sol (Meeus, ch. 11 et 40). Indispensable pour la Lune (jusqu'à 1°).
pub fn topocentric(position: Equatorial, distance_au: f64, local_sidereal_deg: f64, site: &GeoSite) -> Equatorial {
    let phi = site.latitude_deg.to_radians();
    let u = (EARTH_FLATTENING_RATIO * phi.tan()).atan();
    let height = site.altitude_m / EARTH_EQUATORIAL_RADIUS_M;
    let rho_sin = EARTH_FLATTENING_RATIO * u.sin() + height * phi.sin();
    let rho_cos = u.cos() + height * phi.cos();

    let sin_pi = SOLAR_PARALLAX_DEG.to_radians().sin() / distance_au;
    let (sin_h, cos_h) = (local_sidereal_deg - position.ra_deg).to_radians().sin_cos();
    let (sin_d, cos_d) = position.dec_deg.to_radians().sin_cos();

    let denominator = cos_d - rho_cos * sin_pi * cos_h;
    let delta_ra = (-rho_cos * sin_pi * sin_h).atan2(denominator);
    let dec = ((sin_d - rho_sin * sin_pi) * delta_ra.cos()).atan2(denominator);
    Equatorial {
        ra_deg: normalize_degrees(position.ra_deg + delta_ra.to_degrees()),
        dec_deg: dec.to_degrees(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dms(d: f64, m: f64, s: f64) -> f64 {
        d.signum() * (d.abs() + m / 60.0 + s / 3600.0)
    }

    #[test]
    fn meeus_example_13b() {
        // Vénus vue de Washington, 1987 avril 10 19h21m UT :
        // A = 68.0337° (compté depuis le sud), h = 15.1249°.
        let venus = Equatorial {
            ra_deg: 15.0 * dms(23.0, 9.0, 16.641),
            dec_deg: -dms(6.0, 43.0, 11.61),
        };
        let theta0 = 15.0 * dms(8.0, 34.0, 56.853);
        let longitude_east = -dms(77.0, 3.0, 56.0);
        let lst = normalize_degrees(theta0 + longitude_east);
        let hor = equatorial_to_horizontal(venus, lst, dms(38.0, 55.0, 17.0));
        assert!((normalize_degrees(hor.azimuth_deg - 180.0) - 68.0337).abs() < 1e-3);
        assert!((hor.altitude_deg - 15.1249).abs() < 1e-3);
    }

    #[test]
    fn cardinal_azimuths() {
        // À l'équateur, une étoile de déclinaison 0 se lève plein est (H = −90°).
        let rising = equatorial_to_horizontal(Equatorial { ra_deg: 90.0, dec_deg: 0.0 }, 0.0, 0.0);
        assert!((rising.azimuth_deg - 90.0).abs() < 1e-9);
        assert!(rising.altitude_deg.abs() < 1e-9);
        let pole = equatorial_to_horizontal(Equatorial { ra_deg: 0.0, dec_deg: 89.999 }, 123.0, 48.0);
        assert!((pole.altitude_deg - 48.0).abs() < 0.01);
    }

    #[test]
    fn separation_and_refraction() {
        let a = Equatorial { ra_deg: 10.0, dec_deg: 0.0 };
        let b = Equatorial { ra_deg: 40.0, dec_deg: 0.0 };
        assert!((angular_separation_deg(a, b) - 30.0).abs() < 1e-9);
        // Réfraction à l'horizon ≈ 29′ (Sæmundsson), nulle au zénith.
        assert!((refraction_deg(0.0) * 60.0 - 28.9).abs() < 0.5);
        assert_eq!(refraction_deg(90.0), 0.0);
        assert_eq!(refraction_deg(-5.0), 0.0);
    }

    #[test]
    fn meeus_example_40a() {
        // Mars, 2003 août 28 3h17m UT, Palomar : α' = 22h38m08.54s, δ' = −15°46′30.0″.
        let site = GeoSite {
            latitude_deg: dms(33.0, 21.0, 22.0),
            longitude_deg: -15.0 * dms(7.0, 47.0, 27.0),
            altitude_m: 1706.0,
        };
        let mars = Equatorial { ra_deg: 339.530_208, dec_deg: -15.771_083 };
        let theta0 = 15.0 * dms(1.0, 40.0, 45.0);
        let lst = normalize_degrees(theta0 + site.longitude_deg);
        let topo = topocentric(mars, 0.37276, lst, &site);
        assert!((topo.ra_deg - 15.0 * dms(22.0, 38.0, 8.54)).abs() < 1e-4);
        assert!((topo.dec_deg + dms(15.0, 46.0, 30.0)).abs() < 1e-4);
    }
}
