use tauri::State;

use crate::dto::SkyDto;
use crate::error::AppResult;
use crate::facades::SkyFacade;

/// `async` : le calcul s'exécute hors du thread principal de la fenêtre.
#[tauri::command]
pub async fn get_sky(facade: State<'_, SkyFacade>, datetime_utc: Option<String>) -> AppResult<SkyDto> {
    facade.get_sky(datetime_utc.as_deref())
}
