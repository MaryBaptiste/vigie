import type { Observer } from "./observer";

/** Miroirs de `SkyDto` et de ses sous-structures (src-tauri/src/dto/sky_dto.rs). */

export type SkyState =
  | "DAY"
  | "CIVIL TWILIGHT"
  | "NAUTICAL TWILIGHT"
  | "ASTRONOMICAL TWILIGHT"
  | "NIGHT";

export type BodyType = "SUN" | "MOON" | "PLANET" | "COMET";

export interface Horizontal {
  /** Depuis le nord, vers l'est. */
  azimuthDeg: number;
  altitudeDeg: number;
}

export interface SkyStar extends Horizontal {
  /** Numéro HR (Bright Star Catalog). */
  id: number;
  name: string | null;
  magnitude: number;
  colorIndex: number | null;
}

export interface SkySegment {
  id: number;
  from: Horizontal;
  to: Horizontal;
}

export interface SkyConstellation {
  abbreviation: string;
  latinName: string;
  frenchName: string;
  segments: SkySegment[];
  label: Horizontal | null;
}

export interface SkyBody extends Horizontal {
  id: string;
  bodyType: BodyType;
  name: string;
  magnitude: number;
  illumination: number | null;
  waxing: boolean | null;
}

export interface Sky {
  /** ISO 8601, UTC. */
  timestamp: string;
  localSiderealTimeHours: number;
  observer: Observer;
  skyState: SkyState;
  sunAltitudeDeg: number;
  stars: SkyStar[];
  constellations: SkyConstellation[];
  bodies: SkyBody[];
}
