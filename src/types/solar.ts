/** Miroir de `TerminatorPointDto` (src-tauri/src/dto/solar_dto.rs). */
export interface TerminatorPoint {
  latitudeDeg: number;
  longitudeDeg: number;
  /** `true` côté lever du soleil, `false` côté coucher. */
  sunrise: boolean;
}

/** Miroir de `SolarDto`. */
export interface SolarState {
  /** Horodatage UTC du calcul, ISO 8601. */
  timestamp: string;
  subsolarLatitudeDeg: number;
  subsolarLongitudeDeg: number;
  /** Ordonné par longitude croissante, de −180° à +180°. */
  terminator: TerminatorPoint[];
}
