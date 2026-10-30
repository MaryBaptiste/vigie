use chrono::Utc;

use crate::dto::SolarDto;
use crate::error::AppResult;
use crate::facades::solar_math;
use crate::mappers::SolarMapper;
use crate::models::SolarState;

/// Pas d'échantillonnage du terminateur, en degrés de longitude.
const TERMINATOR_STEP_DEG: f64 = 1.0;

#[derive(Default)]
pub struct SolarFacade;

impl SolarFacade {
    pub fn get_solar_state(&self) -> AppResult<SolarDto> {
        let now = Utc::now();
        let subsolar = solar_math::subsolar_point(now);
        let state = SolarState {
            computed_at: now,
            subsolar,
            terminator: solar_math::terminator(subsolar, TERMINATOR_STEP_DEG),
        };
        Ok(SolarMapper::to_dto(&state))
    }
}
