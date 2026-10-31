import type { Observer } from "../types/observer";

/** Villes prédéfinies, dont trois dans l'hémisphère sud. */
export const PRESET_CITIES: readonly Observer[] = [
  { label: "Paris", latitudeDeg: 48.8566, longitudeDeg: 2.3522, altitudeM: 35 },
  { label: "Lyon", latitudeDeg: 45.764, longitudeDeg: 4.8357, altitudeM: 173 },
  { label: "Marseille", latitudeDeg: 43.2965, longitudeDeg: 5.3698, altitudeM: 12 },
  { label: "Montréal", latitudeDeg: 45.5019, longitudeDeg: -73.5674, altitudeM: 36 },
  { label: "Tokyo", latitudeDeg: 35.6762, longitudeDeg: 139.6503, altitudeM: 40 },
  { label: "Sydney", latitudeDeg: -33.8688, longitudeDeg: 151.2093, altitudeM: 58 },
  { label: "Santiago", latitudeDeg: -33.4489, longitudeDeg: -70.6693, altitudeM: 570 },
];
