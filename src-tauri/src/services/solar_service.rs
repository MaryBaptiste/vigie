use tauri::State;

use crate::dto::SolarDto;
use crate::error::AppResult;
use crate::facades::SolarFacade;

#[tauri::command]
pub fn get_solar_state(facade: State<'_, SolarFacade>) -> AppResult<SolarDto> {
    facade.get_solar_state()
}
