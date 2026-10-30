use chrono::{DateTime, Utc};

use super::GeodeticPosition;

/// Position de l'objet à un instant donné de sa trajectoire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrajectoryPoint {
    pub time: DateTime<Utc>,
    pub position: GeodeticPosition,
}

/// Trajectoire échantillonnée autour de l'instant du calcul.
#[derive(Debug, Clone, PartialEq)]
pub struct Trajectory {
    pub norad_id: u32,
    pub computed_at: DateTime<Utc>,
    pub points: Vec<TrajectoryPoint>,
}
