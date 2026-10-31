//! Calculs astronomiques purs : sans état ni I/O, testés contre des valeurs
//! de référence (exemples résolus de Meeus, *Astronomical Algorithms*, 2e éd.,
//! éphémérides JPL Horizons et Skyfield).
//!
//! Conventions : angles en degrés dans les API publiques, azimut compté depuis
//! le nord vers l'est, jours juliens UT (`jd`) ou TT (`jde`).

pub mod angles;
pub mod comet;
pub mod constellation;
pub mod coordinates;
pub mod epoch;
pub mod linalg;
pub mod moon;
mod moon_tables;
pub mod nutation;
pub mod planets;
pub mod precession;
pub mod rise_set;
pub mod sidereal;
pub mod stars;
pub mod sun;
pub mod time;
pub mod vsop87;
mod vsop87_data;

pub use coordinates::{Equatorial, GeoSite, Horizontal};
pub use epoch::Epoch;
pub use planets::Planet;

