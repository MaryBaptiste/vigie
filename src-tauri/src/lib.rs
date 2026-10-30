mod clients;
mod dto;
mod error;
mod facades;
mod mappers;
mod models;
mod services;

use tauri::Manager;

use clients::CelestrakClient;
use facades::{SolarFacade, SpaceObjectFacade, TelemetryFacade};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(SpaceObjectFacade);
            app.manage(SolarFacade);
            app.manage(TelemetryFacade::new(CelestrakClient::new()?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            services::space_object_service::get_space_objects,
            services::telemetry_service::get_telemetry,
            services::trajectory_service::get_trajectory,
            services::solar_service::get_solar_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
