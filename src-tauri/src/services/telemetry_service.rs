use tauri::State;

use crate::dto::TelemetryDto;
use crate::error::AppResult;
use crate::facades::TelemetryFacade;

#[tauri::command]
pub async fn get_telemetry(
    facade: State<'_, TelemetryFacade>,
    norad_id: u32,
) -> AppResult<TelemetryDto> {
    facade.get_telemetry(norad_id).await
}
