import type { Horizontal } from "../types/sky";

const RAD = Math.PI / 180;

/**
 * Repère local ENU : x vers l'est, y vers le nord, z vers le zénith.
 * Écrit le vecteur unitaire de (az, h) dans `out` à partir de `offset`.
 */
export function writeEnu(position: Horizontal, out: Float32Array, offset: number): void {
  const az = position.azimuthDeg * RAD;
  const alt = position.altitudeDeg * RAD;
  const cosAlt = Math.cos(alt);
  out[offset] = cosAlt * Math.sin(az);
  out[offset + 1] = cosAlt * Math.cos(az);
  out[offset + 2] = Math.sin(alt);
}

export function enuToHorizontal(e: number, n: number, u: number): Horizontal {
  const azimuth = Math.atan2(e, n) / RAD;
  return {
    azimuthDeg: (azimuth + 360) % 360,
    altitudeDeg: Math.asin(Math.max(-1, Math.min(1, u))) / RAD,
  };
}

/**
 * Projection stéréographique azimutale : zénith au centre, horizon sur le
 * cercle unité, nord en haut et est à gauche (carte du ciel tenue au-dessus
 * de la tête). Renvoie des coordonnées « écran » (y vers le bas).
 */
export function stereographic(e: number, n: number, u: number): [x: number, y: number] {
  const k = 1 / (1 + Math.max(u, -0.999));
  return [-e * k, -n * k];
}

/** Rayon projeté d'un cercle de hauteur donnée (0 → 1, 90° → 0). */
export function altitudeRadius(altitudeDeg: number): number {
  return Math.tan(((90 - altitudeDeg) / 2) * RAD);
}

export type Enu = readonly [e: number, n: number, u: number];

/** Point où le grand cercle [a, b] coupe l'horizon (a au-dessus, b en dessous). */
export function horizonCrossing(a: Enu, b: Enu): [number, number, number] {
  const t = a[2] / (a[2] - b[2]);
  const e = a[0] + t * (b[0] - a[0]);
  const n = a[1] + t * (b[1] - a[1]);
  const norm = Math.hypot(e, n) || 1;
  return [e / norm, n / norm, 0];
}
