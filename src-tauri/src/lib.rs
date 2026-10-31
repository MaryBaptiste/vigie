mod astronomy;
mod catalogs;
mod clients;
mod dto;
mod error;
mod facades;
mod mappers;
mod models;
mod services;

use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_store::StoreExt;

use catalogs::Catalogs;
use clients::{CelestrakClient, MpcClient};
use facades::{
    CelestialBodyFacade, CometCatalog, ObserverFacade, SkyFacade, SolarFacade, SpaceObjectFacade, StoreObserverStorage,
    TelemetryFacade,
};

const OBSERVER_STORE: &str = "observer.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Suivi d'objets en orbite.
            app.manage(SpaceObjectFacade);
            app.manage(SolarFacade);
            app.manage(TelemetryFacade::new(CelestrakClient::new()?));

            // Planétarium.
            let catalogs = Arc::new(Catalogs::load()?);
            let storage = StoreObserverStorage::new(app.store(OBSERVER_STORE)?);
            let observer = ObserverFacade::new(Arc::new(storage));
            let comets = Arc::new(CometCatalog::new(
                MpcClient::new()?,
                app.path().app_cache_dir()?.join("comets"),
            ));

            app.manage(SkyFacade::new(Arc::clone(&catalogs), observer.clone(), Arc::clone(&comets)));
            app.manage(CelestialBodyFacade::new(catalogs, observer.clone(), comets));
            app.manage(observer);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            services::space_object_service::get_space_objects,
            services::telemetry_service::get_telemetry,
            services::trajectory_service::get_trajectory,
            services::solar_service::get_solar_state,
            services::observer_service::get_observer,
            services::observer_service::set_observer,
            services::sky_service::get_sky,
            services::celestial_body_service::get_visible_bodies,
            services::celestial_body_service::get_comet_catalog_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
