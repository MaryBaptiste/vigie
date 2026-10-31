use tauri::State;

use crate::dto::ObserverDto;
use crate::error::AppResult;
use crate::facades::ObserverFacade;

#[tauri::command]
pub fn get_observer(facade: State<'_, ObserverFacade>) -> AppResult<ObserverDto> {
    Ok(facade.get_observer())
}

#[tauri::command]
pub fn set_observer(
    facade: State<'_, ObserverFacade>,
    lat: f64,
    lon: f64,
    altitude_m: f64,
    label: String,
) -> AppResult<ObserverDto> {
    facade.set_observer(lat, lon, altitude_m, &label)
}
