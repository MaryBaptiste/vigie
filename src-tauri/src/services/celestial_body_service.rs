use tauri::State;

use crate::dto::{CelestialBodyDto, CometCatalogStatusDto};
use crate::error::AppResult;
use crate::facades::CelestialBodyFacade;

#[tauri::command]
pub async fn get_visible_bodies(
    facade: State<'_, CelestialBodyFacade>,
    datetime_utc: Option<String>,
) -> AppResult<Vec<CelestialBodyDto>> {
    facade.get_visible_bodies(datetime_utc.as_deref())
}

#[tauri::command]
pub fn get_comet_catalog_status(facade: State<'_, CelestialBodyFacade>) -> AppResult<CometCatalogStatusDto> {
    Ok(facade.get_comet_catalog_status())
}
