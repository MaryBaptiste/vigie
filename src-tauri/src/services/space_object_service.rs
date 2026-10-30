use tauri::State;

use crate::dto::SpaceObjectDto;
use crate::error::AppResult;
use crate::facades::SpaceObjectFacade;

#[tauri::command]
pub fn get_space_objects(facade: State<'_, SpaceObjectFacade>) -> AppResult<Vec<SpaceObjectDto>> {
    facade.get_space_objects()
}
