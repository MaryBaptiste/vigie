use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::Mutex;

use crate::clients::CelestrakClient;
use crate::dto::{TelemetryDto, TrajectoryDto};
use crate::error::{AppError, AppResult};
use crate::facades::orbit_math;
use crate::mappers::{TelemetryMapper, TrajectoryMapper};
use crate::models::{OrbitalState, Trajectory, TrajectoryPoint};

/// Durée de validité du cache des éléments orbitaux.
const CACHE_TTL: TimeDelta = TimeDelta::hours(12);
/// Délai minimal entre deux tentatives après un échec, pour ne pas solliciter
/// CelesTrak à chaque polling quand le réseau est coupé.
const RETRY_BACKOFF: TimeDelta = TimeDelta::seconds(30);

/// Fenêtre de trajectoire, en nombre de périodes orbitales avant/après l'instant courant.
const TRAJECTORY_PAST_PERIODS: f64 = 0.5;
const TRAJECTORY_FUTURE_PERIODS: f64 = 1.0;
const TRAJECTORY_STEP: TimeDelta = TimeDelta::seconds(30);

struct CachedElements {
    elements: sgp4::Elements,
    constants: sgp4::Constants,
    fetched_at: DateTime<Utc>,
}

struct FetchFailure {
    at: DateTime<Utc>,
    message: String,
}

#[derive(Default)]
struct Cache {
    entries: HashMap<u32, Arc<CachedElements>>,
    failures: HashMap<u32, FetchFailure>,
}

pub struct TelemetryFacade {
    client: CelestrakClient,
    cache: Mutex<Cache>,
}

impl TelemetryFacade {
    pub fn new(client: CelestrakClient) -> Self {
        Self {
            client,
            cache: Mutex::new(Cache::default()),
        }
    }

    pub async fn get_telemetry(&self, norad_id: u32) -> AppResult<TelemetryDto> {
        let cached = self.elements_for(norad_id).await?;
        let state = compute_state(norad_id, &cached, Utc::now())?;
        Ok(TelemetryMapper::to_dto(&state))
    }

    pub async fn get_trajectory(&self, norad_id: u32) -> AppResult<TrajectoryDto> {
        let cached = self.elements_for(norad_id).await?;
        let trajectory = compute_trajectory(norad_id, &cached, Utc::now())?;
        Ok(TrajectoryMapper::to_dto(&trajectory))
    }

    /// Éléments en cache s'ils ont moins de 12 h, sinon rafraîchis depuis
    /// CelesTrak. En cas d'échec, les derniers éléments connus sont réutilisés.
    async fn elements_for(&self, norad_id: u32) -> AppResult<Arc<CachedElements>> {
        let now = Utc::now();
        let mut cache = self.cache.lock().await;
        let existing = cache.entries.get(&norad_id).cloned();

        if let Some(entry) = &existing
            && now - entry.fetched_at < CACHE_TTL
        {
            return Ok(Arc::clone(entry));
        }

        if let Some(failure) = cache.failures.get(&norad_id) {
            let elapsed = now - failure.at;
            if elapsed < RETRY_BACKOFF {
                return existing.ok_or_else(|| AppError::Backoff {
                    retry_in_secs: (RETRY_BACKOFF - elapsed).num_seconds().max(1) as u64,
                    message: failure.message.clone(),
                });
            }
        }

        match self.fetch(norad_id, now).await {
            Ok(entry) => {
                let entry = Arc::new(entry);
                cache.failures.remove(&norad_id);
                cache.entries.insert(norad_id, Arc::clone(&entry));
                Ok(entry)
            }
            Err(err) => {
                cache.failures.insert(
                    norad_id,
                    FetchFailure {
                        at: now,
                        message: err.to_string(),
                    },
                );
                existing.ok_or(err)
            }
        }
    }

    async fn fetch(&self, norad_id: u32, now: DateTime<Utc>) -> AppResult<CachedElements> {
        let elements = self
            .client
            .fetch_gp(norad_id)
            .await?
            .ok_or(AppError::NotFound(norad_id))?;
        let constants = sgp4::Constants::from_elements(&elements)
            .map_err(|e| AppError::Propagation(e.to_string()))?;
        Ok(CachedElements {
            elements,
            constants,
            fetched_at: now,
        })
    }
}

/// Propagation SGP4 à l'instant `at`. Renvoie aussi les minutes depuis l'époque.
fn propagate(
    cached: &CachedElements,
    at: DateTime<Utc>,
) -> AppResult<(sgp4::Prediction, sgp4::MinutesSinceEpoch)> {
    let minutes = cached
        .elements
        .datetime_to_minutes_since_epoch(&at.naive_utc())
        .map_err(|e| AppError::Propagation(e.to_string()))?;
    let prediction = cached
        .constants
        .propagate(minutes)
        .map_err(|e| AppError::Propagation(e.to_string()))?;
    Ok((prediction, minutes))
}

/// Trajectoire de `-TRAJECTORY_PAST_PERIODS` à `+TRAJECTORY_FUTURE_PERIODS`
/// périodes autour de `now`, en coordonnées terrestres (trace au sol en altitude).
fn compute_trajectory(
    norad_id: u32,
    cached: &CachedElements,
    now: DateTime<Utc>,
) -> AppResult<Trajectory> {
    let period_min =
        orbit_math::orbit_shape(cached.elements.mean_motion, cached.elements.eccentricity)
            .period_min;
    let period_s = period_min * 60.0;
    let start_offset_s = -(period_s * TRAJECTORY_PAST_PERIODS);
    let end_offset_s = period_s * TRAJECTORY_FUTURE_PERIODS;
    let step_s = TRAJECTORY_STEP.num_seconds() as f64;
    let count = ((end_offset_s - start_offset_s) / step_s).ceil() as i64;

    let points = (0..=count)
        .map(|i| {
            let offset_ms = ((start_offset_s + i as f64 * step_s) * 1000.0) as i64;
            let time = now + TimeDelta::milliseconds(offset_ms);
            let (prediction, _) = propagate(cached, time)?;
            Ok(TrajectoryPoint {
                time,
                position: orbit_math::teme_to_geodetic(prediction.position, time),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Trajectory {
        norad_id,
        computed_at: now,
        points,
    })
}

fn compute_state(
    norad_id: u32,
    cached: &CachedElements,
    now: DateTime<Utc>,
) -> AppResult<OrbitalState> {
    let elements = &cached.elements;
    let (prediction, minutes) = propagate(cached, now)?;

    let shape = orbit_math::orbit_shape(elements.mean_motion, elements.eccentricity);
    let tle_epoch = elements.datetime.and_utc();

    Ok(OrbitalState {
        norad_id,
        computed_at: now,
        position: orbit_math::teme_to_geodetic(prediction.position, now),
        speed_km_s: orbit_math::norm(prediction.velocity),
        inclination_deg: elements.inclination,
        period_min: shape.period_min,
        apogee_km: shape.apogee_km,
        perigee_km: shape.perigee_km,
        eccentricity: elements.eccentricity,
        revolution_number: orbit_math::revolution_number(
            elements.revolution_number,
            elements.argument_of_perigee,
            elements.mean_anomaly,
            minutes.0,
            shape.period_min,
        ),
        tle_epoch,
        tle_age_hours: (now - tle_epoch).num_milliseconds() as f64 / 3_600_000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISS_OMM: &str = r#"{"OBJECT_NAME":"ISS (ZARYA)","OBJECT_ID":"1998-067A","EPOCH":"2026-10-29T13:31:42.192192","MEAN_MOTION":15.49100842,"ECCENTRICITY":0.00070497,"INCLINATION":51.6333,"RA_OF_ASC_NODE":355.1294,"ARG_OF_PERICENTER":324.3141,"MEAN_ANOMALY":35.7375,"EPHEMERIS_TYPE":0,"CLASSIFICATION_TYPE":"U","NORAD_CAT_ID":25544,"ELEMENT_SET_NO":999,"REV_AT_EPOCH":59258,"BSTAR":0.00010764701,"MEAN_MOTION_DOT":5.507e-5,"MEAN_MOTION_DDOT":0}"#;

    fn iss_cache() -> (CachedElements, DateTime<Utc>) {
        let elements: sgp4::Elements = serde_json::from_str(ISS_OMM).unwrap();
        let constants = sgp4::Constants::from_elements(&elements).unwrap();
        let epoch = elements.datetime.and_utc();
        let cached = CachedElements {
            elements,
            constants,
            fetched_at: epoch,
        };
        (cached, epoch)
    }

    #[test]
    fn iss_trajectory_spans_window_around_now() {
        let (cached, epoch) = iss_cache();
        let now = epoch + TimeDelta::hours(3);
        let trajectory = compute_trajectory(25544, &cached, now).unwrap();

        let first = trajectory.points.first().unwrap().time;
        let last = trajectory.points.last().unwrap().time;
        // ISS : période ≈ 93 min → −46 min / +93 min, un point toutes les 30 s.
        assert!(((now - first).num_seconds() - 46 * 60).abs() < 60);
        assert!(((last - now).num_seconds() - 93 * 60).abs() < 60);
        assert!((275..=285).contains(&trajectory.points.len()));
        assert!(trajectory.points.windows(2).all(|w| w[0].time < w[1].time));
        for point in &trajectory.points {
            assert!((395.0..435.0).contains(&point.position.altitude_km));
            assert!(point.position.latitude_deg.abs() <= 52.0);
        }
    }

    #[test]
    fn iss_state_is_physically_plausible() {
        let (cached, epoch) = iss_cache();

        for hours in [0, 6, 24] {
            let now = epoch + TimeDelta::hours(hours);
            let state = compute_state(25544, &cached, now).unwrap();
            assert!((395.0..435.0).contains(&state.position.altitude_km), "{state:?}");
            assert!((7.6..7.72).contains(&state.speed_km_s), "{state:?}");
            assert!(state.position.latitude_deg.abs() <= 52.0, "{state:?}");
            assert!((state.tle_age_hours - hours as f64).abs() < 1e-6);
            assert!(state.revolution_number >= 59258);
        }
    }
}
