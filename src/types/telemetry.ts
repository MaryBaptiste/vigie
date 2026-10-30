/** Miroir de `TelemetryDto` (src-tauri/src/dto/telemetry_dto.rs). */
export interface Telemetry {
  noradId: number;
  /** Horodatage UTC du calcul, ISO 8601. */
  timestamp: string;
  latitudeDeg: number;
  longitudeDeg: number;
  altitudeKm: number;
  speedKmS: number;
  inclinationDeg: number;
  periodMin: number;
  apogeeKm: number;
  perigeeKm: number;
  eccentricity: number;
  revolutionNumber: number;
  /** Époque du TLE, ISO 8601 UTC. */
  tleEpoch: string;
  tleAgeHours: number;
}
