use tauri::State;

use crate::dto::TrajectoryDto;
use crate::error::AppResult;
use crate::facades::TelemetryFacade;

#[tauri::command]
pub async fn get_trajectory(
    facade: State<'_, TelemetryFacade>,
    norad_id: u32,
) -> AppResult<TrajectoryDto> {
    facade.get_trajectory(norad_id).await
}
