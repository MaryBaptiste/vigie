/** Miroir de `TrajectoryPointDto` (src-tauri/src/dto/trajectory_dto.rs). */
export interface TrajectoryPoint {
  /** Instant du point, ISO 8601 UTC. */
  timestamp: string;
  latitudeDeg: number;
  longitudeDeg: number;
  altitudeKm: number;
}

/** Miroir de `TrajectoryDto`. */
export interface Trajectory {
  noradId: number;
  /** Horodatage UTC du calcul, ISO 8601. */
  computedAt: string;
  points: TrajectoryPoint[];
}
