//! Lever, passage au méridien et coucher, à partir d'une courbe de hauteur
//! échantillonnée : les passages par la hauteur de référence h0 sont
//! interpolés linéairement, le maximum par une parabole.

/// Hauteur de référence d'une étoile ou planète : réfraction à l'horizon (34′).
pub const STAR_H0_DEG: f64 = -0.5667;
/// Hauteur de référence du Soleil : réfraction et demi-diamètre (50′).
pub const SUN_H0_DEG: f64 = -0.8333;

/// Hauteur de référence de la Lune en coordonnées topocentriques : bord
/// supérieur sur l'horizon.
pub fn moon_h0_deg(semidiameter_deg: f64) -> f64 {
    STAR_H0_DEG - semidiameter_deg
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AltitudeSample {
    pub jd: f64,
    pub altitude_deg: f64,
}

/// Échantillonne `altitude(jd)` de `start` à `end` inclus, par pas de `step` jours.
pub fn sample_altitudes(start: f64, end: f64, step: f64, mut altitude: impl FnMut(f64) -> f64) -> Vec<AltitudeSample> {
    let count = ((end - start) / step).round() as usize;
    (0..=count)
        .map(|i| {
            let jd = start + i as f64 * step;
            AltitudeSample { jd, altitude_deg: altitude(jd) }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crossing {
    pub jd: f64,
    pub rising: bool,
}

/// Passages par la hauteur `h0`, dans l'ordre chronologique.
pub fn crossings(samples: &[AltitudeSample], h0: f64) -> Vec<Crossing> {
    samples
        .windows(2)
        .filter_map(|pair| {
            let (a, b) = (pair[0].altitude_deg - h0, pair[1].altitude_deg - h0);
            if (a < 0.0) == (b < 0.0) {
                return None;
            }
            let fraction = a / (a - b);
            Some(Crossing {
                jd: pair[0].jd + fraction * (pair[1].jd - pair[0].jd),
                rising: b > a,
            })
        })
        .collect()
}

/// Hauteur interpolée linéairement à l'instant `jd`.
pub fn altitude_at(samples: &[AltitudeSample], jd: f64) -> Option<f64> {
    samples.windows(2).find(|p| p[0].jd <= jd && jd <= p[1].jd).map(|p| {
        let fraction = (jd - p[0].jd) / (p[1].jd - p[0].jd);
        p[0].altitude_deg + fraction * (p[1].altitude_deg - p[0].altitude_deg)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RiseTransitSet {
    pub rise: Option<f64>,
    pub transit: Option<f64>,
    pub transit_altitude_deg: Option<f64>,
    pub set: Option<f64>,
}

/// Premiers lever, culmination et coucher de la fenêtre échantillonnée.
pub fn rise_transit_set(samples: &[AltitudeSample], h0: f64) -> RiseTransitSet {
    let all = crossings(samples, h0);
    let culmination = samples.windows(3).find_map(|w| {
        let (a, b, c) = (w[0].altitude_deg, w[1].altitude_deg, w[2].altitude_deg);
        if !(b >= a && b > c) {
            return None;
        }
        // Sommet de la parabole passant par les trois points (pas constant).
        let curvature = a - 2.0 * b + c;
        let offset = if curvature.abs() > 1e-12 { 0.5 * (a - c) / curvature } else { 0.0 };
        let step = w[1].jd - w[0].jd;
        Some((w[1].jd + offset * step, b - 0.25 * (a - c) * offset))
    });
    RiseTransitSet {
        rise: all.iter().find(|c| c.rising).map(|c| c.jd),
        transit: culmination.map(|(jd, _)| jd),
        transit_altitude_deg: culmination.map(|(_, alt)| alt),
        set: all.iter().find(|c| !c.rising).map(|c| c.jd),
    }
}

/// Hauteur maximale entre `from` et `to`, bornes interpolées comprises.
pub fn max_altitude_between(samples: &[AltitudeSample], from: f64, to: f64) -> Option<f64> {
    let inner = samples.iter().filter(|s| s.jd >= from && s.jd <= to).map(|s| s.altitude_deg);
    let bounds = [altitude_at(samples, from), altitude_at(samples, to)].into_iter().flatten();
    inner.chain(bounds).reduce(f64::max)
}

/// Nuit de référence : en cours si le Soleil est couché, sinon la prochaine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NightWindow {
    pub sunset: Option<f64>,
    pub sunrise: Option<f64>,
    /// Bornes de la nuit, ou fenêtre de 24 h centrée sur le minimum du Soleil
    /// quand il ne se couche ou ne se lève pas (régions polaires).
    pub start: f64,
    pub end: f64,
}

impl NightWindow {
    pub fn middle(&self) -> f64 {
        (self.start + self.end) / 2.0
    }
}

/// Détermine la nuit de référence à partir de la hauteur du Soleil, échantillonnée
/// au moins de `now − 1 j` à `now + 1.5 j`.
pub fn night_window(sun: &[AltitudeSample], now: f64) -> NightWindow {
    let events = crossings(sun, SUN_H0_DEG);
    let is_night = altitude_at(sun, now).is_some_and(|alt| alt < SUN_H0_DEG);
    let (sunset, sunrise) = if is_night {
        let sunset = events.iter().rev().find(|c| !c.rising && c.jd <= now).map(|c| c.jd);
        let sunrise = events.iter().find(|c| c.rising && c.jd > now).map(|c| c.jd);
        (sunset, sunrise)
    } else {
        let sunset = events.iter().find(|c| !c.rising && c.jd > now).map(|c| c.jd);
        let sunrise = sunset.and_then(|s| events.iter().find(|c| c.rising && c.jd > s).map(|c| c.jd));
        (sunset, sunrise)
    };

    if let (Some(start), Some(end)) = (sunset, sunrise) {
        return NightWindow { sunset, sunrise, start, end };
    }
    let (from, to) = if is_night { (now - 0.5, now + 0.5) } else { (now, now + 1.0) };
    let lowest = sun
        .iter()
        .filter(|s| s.jd >= from && s.jd <= to)
        .min_by(|a, b| a.altitude_deg.total_cmp(&b.altitude_deg))
        .map_or(now, |s| s.jd);
    NightWindow { sunset, sunrise, start: lowest - 0.5, end: lowest + 0.5 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::coordinates::GeoSite;
    use crate::astronomy::epoch::Epoch;
    use crate::astronomy::planets::{Planet, planet_position};
    use crate::astronomy::sun::sun_position;

    fn sinusoid(start: f64) -> Vec<AltitudeSample> {
        // Hauteur fictive : culmine à 40° à start + 0.5, période d'un jour.
        sample_altitudes(start, start + 1.0, 1.0 / 144.0, |jd| {
            40.0 * (std::f64::consts::TAU * (jd - start - 0.25)).sin()
        })
    }

    #[test]
    fn finds_events_on_a_sinusoid() {
        let events = rise_transit_set(&sinusoid(100.0), 0.0);
        assert!((events.rise.unwrap() - 100.25).abs() < 1e-3);
        assert!((events.transit.unwrap() - 100.5).abs() < 1e-3);
        assert!((events.transit_altitude_deg.unwrap() - 40.0).abs() < 0.01);
        assert!((events.set.unwrap() - 100.75).abs() < 1e-3);
    }

    #[test]
    fn meeus_example_15a() {
        // Vénus à Boston, 1988 mars 20 : lever 12h25m, passage 19h41m, coucher 2h55m UT.
        let boston = GeoSite { latitude_deg: 42.3333, longitude_deg: -71.0833, altitude_m: 0.0 };
        let start = 2_447_240.5;
        let samples = sample_altitudes(start, start + 1.0, 10.0 / 1440.0, |jd| {
            let epoch = Epoch::from_julian_day(jd);
            epoch.horizontal(planet_position(Planet::Venus, &epoch).equatorial, &boston).altitude_deg
        });
        let events = rise_transit_set(&samples, STAR_H0_DEG);
        let minutes = |jd: Option<f64>| (jd.unwrap() - start) * 1440.0;
        let tolerance = 2.0;
        assert!((minutes(events.rise) - (12.0 * 60.0 + 25.0)).abs() < tolerance);
        assert!((minutes(events.transit) - (19.0 * 60.0 + 41.0)).abs() < tolerance);
        assert!((minutes(events.set) - (2.0 * 60.0 + 55.0)).abs() < tolerance);
    }

    #[test]
    fn night_window_in_paris() {
        // 2026-10-30, Paris (Skyfield) : coucher 16h33m38s UT, lever le 31 à 6h35m45s UT.
        let paris = GeoSite { latitude_deg: 48.8566, longitude_deg: 2.3522, altitude_m: 0.0 };
        let noon = 2_461_344.0;
        let sun = sample_altitudes(noon - 1.0, noon + 1.5, 10.0 / 1440.0, |jd| {
            let epoch = Epoch::from_julian_day(jd);
            epoch.horizontal(sun_position(&epoch).equatorial, &paris).altitude_deg
        });
        let night = night_window(&sun, noon);
        let hour = |jd: f64| (jd - 2_461_343.5) * 24.0;
        let tolerance = 2.0 / 60.0;
        assert!((hour(night.start) - 16.561).abs() < tolerance, "coucher {}", hour(night.start));
        assert!((hour(night.end) - 30.596).abs() < tolerance, "lever {}", hour(night.end));
        // Pendant la nuit, la même nuit est renvoyée.
        assert_eq!(night_window(&sun, noon + 0.4).sunset, night.sunset);
    }

    #[test]
    fn polar_night_falls_back_to_a_day_window() {
        let samples = sample_altitudes(0.0, 2.5, 1.0 / 144.0, |jd| -10.0 + 5.0 * (std::f64::consts::TAU * jd).cos());
        let night = night_window(&samples, 1.0);
        assert!(night.sunset.is_none());
        assert!((night.end - night.start - 1.0).abs() < 1e-9);
    }
}
