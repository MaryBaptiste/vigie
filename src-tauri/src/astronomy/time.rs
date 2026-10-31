use chrono::{DateTime, TimeZone, Utc};

use super::angles::horner;

/// Jour julien de l'époque J2000.0 (2000-01-01 12:00 TT).
pub const J2000: f64 = 2_451_545.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;
const MS_PER_DAY: f64 = 86_400_000.0;
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const DAYS_PER_CENTURY: f64 = 36_525.0;
pub const DAYS_PER_YEAR: f64 = 365.25;

/// Jour julien (échelle UT) d'un instant UTC. UTC et UT1 diffèrent de moins
/// d'une seconde, ce qui est négligeable ici.
pub fn julian_day(instant: &DateTime<Utc>) -> f64 {
    UNIX_EPOCH_JD + instant.timestamp_millis() as f64 / MS_PER_DAY
}

pub fn datetime_from_julian_day(jd: f64) -> DateTime<Utc> {
    let ms = ((jd - UNIX_EPOCH_JD) * MS_PER_DAY).round() as i64;
    Utc.timestamp_millis_opt(ms).single().unwrap_or_default()
}

/// Siècles juliens écoulés depuis J2000.0.
pub fn julian_centuries(jd: f64) -> f64 {
    (jd - J2000) / DAYS_PER_CENTURY
}

pub fn decimal_year(jd: f64) -> f64 {
    2000.0 + (jd - J2000) / DAYS_PER_YEAR
}

/// ΔT = TT − UT en secondes : polynômes d'Espenak & Meeus (2006), valables
/// de 1900 à 2150, et parabole de Morrison & Stephenson au-delà.
pub fn delta_t_seconds(year: f64) -> f64 {
    let long_term = |y: f64| {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    };
    match year {
        y if y < 1900.0 => long_term(y),
        y if y < 1920.0 => horner(y - 1900.0, &[-2.79, 1.494119, -0.0598939, 0.0061966, -0.000197]),
        y if y < 1941.0 => horner(y - 1920.0, &[21.20, 0.84493, -0.076100, 0.0020936]),
        y if y < 1961.0 => horner(y - 1950.0, &[29.07, 0.407, -1.0 / 233.0, 1.0 / 2547.0]),
        y if y < 1986.0 => horner(y - 1975.0, &[45.45, 1.067, -1.0 / 260.0, -1.0 / 718.0]),
        y if y < 2005.0 => horner(
            y - 2000.0,
            &[63.86, 0.3345, -0.060374, 0.0017275, 0.000651814, 0.00002373599],
        ),
        y if y < 2050.0 => horner(y - 2000.0, &[62.92, 0.32217, 0.005589]),
        y if y < 2150.0 => long_term(y) - 0.5628 * (2150.0 - y),
        y => long_term(y),
    }
}

/// Jour julien des éphémérides (TT) correspondant à un jour julien UT.
pub fn terrestrial_time(jd: f64) -> f64 {
    jd + delta_t_seconds(decimal_year(jd)) / SECONDS_PER_DAY
}

/// Jour julien UT correspondant à un jour julien TT (inverse de `terrestrial_time`).
#[cfg(test)]
pub fn universal_time(jde: f64) -> f64 {
    jde - delta_t_seconds(decimal_year(jde)) / SECONDS_PER_DAY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn julian_day_of_known_dates() {
        // Meeus, exemple 7.a : 1957 octobre 4.81 = JD 2436116.31.
        let sputnik = Utc.with_ymd_and_hms(1957, 10, 4, 19, 26, 24).unwrap();
        assert!((julian_day(&sputnik) - 2_436_116.31).abs() < 1e-6);
        let j2000 = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(julian_day(&j2000), J2000);
    }

    #[test]
    fn round_trips_through_datetime() {
        let instant = Utc.with_ymd_and_hms(2026, 10, 30, 20, 44, 12).unwrap();
        assert_eq!(datetime_from_julian_day(julian_day(&instant)), instant);
    }

    #[test]
    fn delta_t_matches_published_values() {
        // Valeurs observées (IERS) : 63.8 s en 2000, 69.4 s en 2020.
        assert!((delta_t_seconds(2000.0) - 63.86).abs() < 0.1);
        assert!((delta_t_seconds(2020.0) - 69.4).abs() < 2.5);
        assert!((delta_t_seconds(1950.0) - 29.07).abs() < 0.1);
    }

    #[test]
    fn universal_time_inverts_terrestrial_time() {
        let jd = 2_461_344.333_333;
        assert!((universal_time(terrestrial_time(jd)) - jd).abs() < 1e-9);
    }
}
