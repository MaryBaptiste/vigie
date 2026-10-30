use crate::dto::SpaceObjectDto;
use crate::error::AppResult;
use crate::mappers::SpaceObjectMapper;
use crate::models::SpaceObject;

/// Objets suivis. Numéros NORAD vérifiés sur CelesTrak le 2026-10-30
/// (voir le test ignoré `catalogue_ids_exist_on_celestrak`).
pub const CATALOGUE: [SpaceObject; 4] = [
    SpaceObject {
        norad_id: 25544,
        name: "ISS (ZARYA)",
        short_name: "ISS",
        description: "Station spatiale internationale",
    },
    SpaceObject {
        norad_id: 20580,
        name: "Hubble Space Telescope",
        short_name: "HUBBLE",
        description: "Télescope spatial NASA/ESA",
    },
    SpaceObject {
        norad_id: 48274,
        name: "Tiangong (Tianhe)",
        short_name: "TIANGONG",
        description: "Station spatiale chinoise, module central",
    },
    SpaceObject {
        norad_id: 49260,
        name: "Landsat 9",
        short_name: "LANDSAT-9",
        description: "Observation de la Terre, orbite héliosynchrone",
    },
];

#[derive(Default)]
pub struct SpaceObjectFacade;

impl SpaceObjectFacade {
    pub fn get_space_objects(&self) -> AppResult<Vec<SpaceObjectDto>> {
        Ok(SpaceObjectMapper::to_dtos(&CATALOGUE))
    }
}
