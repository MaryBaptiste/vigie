use crate::dto::{BodyTypeDto, CelestialBodyDto, ConstellationRefDto, NightCurveDto, SkyBodyDto, VisibilityDto};
use crate::mappers::{iso, round4};
use crate::models::{BodyKind, BodyReport, CelestialBody, Constellation, MoonPhaseKind, NightInfo, Visibility};

pub struct CelestialBodyMapper;

impl CelestialBodyMapper {
    /// Version allégée pour la voûte céleste.
    pub fn to_sky_dto(body: &CelestialBody) -> SkyBodyDto {
        SkyBodyDto {
            id: body.id.clone(),
            body_type: Self::body_type(body.kind),
            name: body.name.clone(),
            azimuth_deg: round4(body.horizontal.azimuth_deg),
            altitude_deg: round4(body.horizontal.altitude_deg),
            magnitude: body.magnitude,
            illumination: body.illumination,
            waxing: body.waxing,
        }
    }

    pub fn to_dto(report: &BodyReport) -> CelestialBodyDto {
        let body = &report.body;
        let is_moon = body.kind == BodyKind::Moon;
        CelestialBodyDto {
            id: body.id.clone(),
            body_type: Self::body_type(body.kind),
            name: body.name.clone(),
            status: Self::visibility(report.visibility),
            azimuth_deg: round4(body.horizontal.azimuth_deg),
            altitude_deg: round4(body.horizontal.altitude_deg),
            right_ascension_deg: round4(body.equatorial.ra_deg),
            declination_deg: round4(body.equatorial.dec_deg),
            magnitude: body.magnitude,
            distance_au: body.distance_au,
            distance_km: is_moon.then_some(body.distance_au * crate::astronomy::moon::KM_PER_AU),
            heliocentric_distance_au: body.heliocentric_distance_au,
            elongation_deg: body.elongation_deg,
            constellation: report.constellation.map(Self::constellation),
            rise: report.rise.as_ref().map(iso),
            transit: report.transit.as_ref().map(iso),
            set: report.set.as_ref().map(iso),
            transit_altitude_deg: report.transit_altitude_deg,
            max_night_altitude_deg: report.max_night_altitude_deg,
            illumination: body.illumination,
            phase_angle_deg: body.phase_angle_deg,
            waxing: body.waxing,
            phase_name: body.moon_phase.map(|p| Self::phase_name(p).to_string()),
            night: Self::night(&report.night, &report.night_altitudes),
        }
    }

    fn body_type(kind: BodyKind) -> BodyTypeDto {
        match kind {
            BodyKind::Sun => BodyTypeDto::Sun,
            BodyKind::Moon => BodyTypeDto::Moon,
            BodyKind::Planet(_) => BodyTypeDto::Planet,
            BodyKind::Comet => BodyTypeDto::Comet,
        }
    }

    fn visibility(visibility: Visibility) -> VisibilityDto {
        match visibility {
            Visibility::Visible => VisibilityDto::Visible,
            Visibility::BelowHorizon => VisibilityDto::BelowHorizon,
            Visibility::LostInDaylight => VisibilityDto::LostInDaylight,
            Visibility::TooFaint => VisibilityDto::TooFaint,
        }
    }

    fn constellation(constellation: &Constellation) -> ConstellationRefDto {
        ConstellationRefDto {
            abbreviation: constellation.abbreviation.clone(),
            latin_name: constellation.latin_name.clone(),
            french_name: constellation.french_name.clone(),
        }
    }

    fn phase_name(phase: MoonPhaseKind) -> &'static str {
        match phase {
            MoonPhaseKind::New => "NEW MOON",
            MoonPhaseKind::WaxingCrescent => "WAXING CRESCENT",
            MoonPhaseKind::FirstQuarter => "FIRST QUARTER",
            MoonPhaseKind::WaxingGibbous => "WAXING GIBBOUS",
            MoonPhaseKind::Full => "FULL MOON",
            MoonPhaseKind::WaningGibbous => "WANING GIBBOUS",
            MoonPhaseKind::LastQuarter => "LAST QUARTER",
            MoonPhaseKind::WaningCrescent => "WANING CRESCENT",
        }
    }

    fn night(night: &NightInfo, altitudes: &[f64]) -> NightCurveDto {
        NightCurveDto {
            start: iso(&night.start),
            end: iso(&night.end),
            sunset: night.sunset.as_ref().map(iso),
            sunrise: night.sunrise.as_ref().map(iso),
            altitudes: altitudes.iter().map(|a| (a * 100.0).round() / 100.0).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::astronomy::{Equatorial, Horizontal};

    fn moon() -> CelestialBody {
        CelestialBody {
            id: "moon".into(),
            name: "Moon".into(),
            kind: BodyKind::Moon,
            equatorial: Equatorial { ra_deg: 102.091_471_9, dec_deg: 26.63 },
            horizontal: Horizontal { azimuth_deg: 50.357_218, altitude_deg: 1.2 },
            geometric_altitude_deg: 0.8,
            magnitude: -11.9,
            distance_au: 0.002_5,
            heliocentric_distance_au: None,
            elongation_deg: 112.0,
            phase_angle_deg: Some(67.0),
            illumination: Some(0.69),
            waxing: Some(false),
            moon_phase: Some(MoonPhaseKind::WaningGibbous),
        }
    }

    fn report(body: CelestialBody) -> BodyReport<'static> {
        let at = |h| Utc.with_ymd_and_hms(2026, 10, 30, h, 0, 0).unwrap();
        BodyReport {
            body,
            visibility: Visibility::BelowHorizon,
            constellation: None,
            rise: Some(at(19)),
            transit: None,
            transit_altitude_deg: None,
            set: None,
            max_night_altitude_deg: Some(60.123),
            night_altitudes: vec![-10.123_45, 20.0],
            night: NightInfo { start: at(16), end: at(23), sunset: Some(at(16)), sunrise: None },
        }
    }

    #[test]
    fn maps_moon_report() {
        let dto = CelestialBodyMapper::to_dto(&report(moon()));
        assert_eq!(dto.body_type, BodyTypeDto::Moon);
        assert_eq!(dto.status, VisibilityDto::BelowHorizon);
        assert_eq!(dto.right_ascension_deg, 102.0915);
        assert!((dto.distance_km.unwrap() - 373_994.7).abs() < 1.0);
        assert_eq!(dto.rise.as_deref(), Some("2026-10-30T19:00:00Z"));
        assert_eq!(dto.phase_name.as_deref(), Some("WANING GIBBOUS"));
        assert_eq!(dto.night.altitudes, vec![-10.12, 20.0]);
    }

    #[test]
    fn serializes_french_statuses() {
        let json = serde_json::to_value(CelestialBodyMapper::to_dto(&report(moon()))).unwrap();
        assert_eq!(json["status"], "BELOW HORIZON");
        assert_eq!(json["bodyType"], "MOON");
        assert!(json["night"]["sunrise"].is_null());
        assert_eq!(serde_json::to_value(VisibilityDto::LostInDaylight).unwrap(), "LOST IN DAYLIGHT");
    }

    #[test]
    fn planets_have_no_distance_in_km() {
        let mut mars = moon();
        mars.kind = BodyKind::Planet(crate::astronomy::Planet::Mars);
        assert!(CelestialBodyMapper::to_dto(&report(mars)).distance_km.is_none());
    }
}
