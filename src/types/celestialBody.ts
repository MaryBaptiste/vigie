import type { BodyType } from "./sky";

/** Miroirs de `CelestialBodyDto` et `CometCatalogStatusDto`. */

export type Visibility = "VISIBLE" | "BELOW HORIZON" | "LOST IN DAYLIGHT" | "TOO FAINT";

export interface ConstellationRef {
  abbreviation: string;
  latinName: string;
  frenchName: string;
}

export interface NightCurve {
  start: string;
  end: string;
  sunset: string | null;
  sunrise: string | null;
  /** Hauteurs régulièrement espacées de `start` à `end`. */
  altitudes: number[];
}

export interface CelestialBody {
  id: string;
  bodyType: BodyType;
  name: string;
  status: Visibility;
  azimuthDeg: number;
  altitudeDeg: number;
  rightAscensionDeg: number;
  declinationDeg: number;
  magnitude: number;
  distanceAu: number;
  distanceKm: number | null;
  heliocentricDistanceAu: number | null;
  elongationDeg: number;
  constellation: ConstellationRef | null;
  rise: string | null;
  transit: string | null;
  set: string | null;
  transitAltitudeDeg: number | null;
  maxNightAltitudeDeg: number | null;
  illumination: number | null;
  phaseAngleDeg: number | null;
  waxing: boolean | null;
  phaseName: string | null;
  night: NightCurve;
}

export type CometSource = "NETWORK" | "CACHE" | "OFFLINE" | "LOADING";

export interface CometCatalogStatus {
  updatedAt: string | null;
  source: CometSource;
  stale: boolean;
  cometCount: number;
}
