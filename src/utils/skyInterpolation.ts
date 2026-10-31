import type { Sky } from "../types/sky";
import { writeEnu } from "./skyMath";

/**
 * Le backend renvoie des instantanés du ciel ; pour que la voûte tourne de
 * façon fluide, chaque image affichée interpole les directions (vecteurs ENU)
 * entre deux instantanés, appariés par identifiant.
 */
export interface PreparedFrame {
  /** Instant simulé de l'instantané (ms epoch). */
  timeMs: number;
  sky: Sky;
  /** 3 composantes par étoile, dans l'ordre de `sky.stars`. */
  stars: Float32Array;
  /** 6 composantes par segment (début, fin), constellations dans l'ordre. */
  segments: Float32Array;
  /** 3 composantes par constellation ; NaN sans libellé. */
  labels: Float32Array;
  /** 3 composantes par objet du système solaire. */
  bodies: Float32Array;
}

export function prepareFrame(sky: Sky): PreparedFrame {
  const stars = new Float32Array(sky.stars.length * 3);
  sky.stars.forEach((star, i) => writeEnu(star, stars, i * 3));

  const segmentCount = sky.constellations.reduce((n, c) => n + c.segments.length, 0);
  const segments = new Float32Array(segmentCount * 6);
  let s = 0;
  for (const constellation of sky.constellations) {
    for (const segment of constellation.segments) {
      writeEnu(segment.from, segments, s * 6);
      writeEnu(segment.to, segments, s * 6 + 3);
      s++;
    }
  }

  const labels = new Float32Array(sky.constellations.length * 3).fill(Number.NaN);
  sky.constellations.forEach((c, i) => {
    if (c.label) writeEnu(c.label, labels, i * 3);
  });

  const bodies = new Float32Array(sky.bodies.length * 3);
  sky.bodies.forEach((body, i) => writeEnu(body, bodies, i * 3));

  return { timeMs: Date.parse(sky.timestamp), sky, stars, segments, labels, bodies };
}

/** Correspondance des éléments de `to` vers leur indice dans `from` (−1 si absent). */
export interface FramePair {
  from: PreparedFrame | null;
  to: PreparedFrame;
  stars: Int32Array;
  segments: Int32Array;
  labels: Int32Array;
  bodies: Int32Array;
}

function indexMap<T, K>(target: readonly T[], source: readonly T[] | null, key: (item: T) => K): Int32Array {
  const map = new Int32Array(target.length).fill(-1);
  if (!source) return map;
  const positions = new Map<K, number>();
  source.forEach((item, i) => positions.set(key(item), i));
  target.forEach((item, i) => (map[i] = positions.get(key(item)) ?? -1));
  return map;
}

export function pairFrames(from: PreparedFrame | null, to: PreparedFrame): FramePair {
  const flatSegments = (frame: PreparedFrame) => frame.sky.constellations.flatMap((c) => c.segments);
  return {
    from,
    to,
    stars: indexMap(to.sky.stars, from?.sky.stars ?? null, (s) => s.id),
    segments: indexMap(flatSegments(to), from ? flatSegments(from) : null, (s) => s.id),
    labels: indexMap(to.sky.constellations, from?.sky.constellations ?? null, (c) => c.abbreviation),
    bodies: indexMap(to.sky.bodies, from?.sky.bodies ?? null, (b) => b.id),
  };
}

export interface RenderBuffers {
  stars: Float32Array;
  segments: Float32Array;
  labels: Float32Array;
  bodies: Float32Array;
}

export function createBuffers(): RenderBuffers {
  return {
    stars: new Float32Array(0),
    segments: new Float32Array(0),
    labels: new Float32Array(0),
    bodies: new Float32Array(0),
  };
}

function lerpVectors(
  to: Float32Array,
  from: Float32Array | null,
  map: Int32Array,
  stride: number,
  fraction: number,
  out: Float32Array,
): void {
  const vectorsPerItem = stride / 3;
  for (let i = 0; i < map.length; i++) {
    const j = map[i] ?? -1;
    for (let v = 0; v < vectorsPerItem; v++) {
      const o = i * stride + v * 3;
      const b0 = to[o] ?? 0;
      const b1 = to[o + 1] ?? 0;
      const b2 = to[o + 2] ?? 0;
      if (!from || j < 0 || Number.isNaN(b0)) {
        out[o] = b0;
        out[o + 1] = b1;
        out[o + 2] = b2;
        continue;
      }
      const p = j * stride + v * 3;
      const a0 = from[p] ?? b0;
      const a1 = from[p + 1] ?? b1;
      const a2 = from[p + 2] ?? b2;
      if (Number.isNaN(a0)) {
        out[o] = b0;
        out[o + 1] = b1;
        out[o + 2] = b2;
        continue;
      }
      // fraction = 0 → instantané précédent, 1 → suivant (au-delà : extrapolation).
      const x = a0 + (b0 - a0) * fraction;
      const y = a1 + (b1 - a1) * fraction;
      const z = a2 + (b2 - a2) * fraction;
      const norm = Math.hypot(x, y, z) || 1;
      out[o] = x / norm;
      out[o + 1] = y / norm;
      out[o + 2] = z / norm;
    }
  }
}

function ensureSize(buffer: Float32Array, length: number): Float32Array {
  return buffer.length === length ? buffer : new Float32Array(length);
}

/** Remplit `out` (redimensionné au besoin) avec les directions interpolées. */
export function interpolate(pair: FramePair, fraction: number, out: RenderBuffers): void {
  const { from, to } = pair;
  out.stars = ensureSize(out.stars, to.stars.length);
  out.segments = ensureSize(out.segments, to.segments.length);
  out.labels = ensureSize(out.labels, to.labels.length);
  out.bodies = ensureSize(out.bodies, to.bodies.length);
  lerpVectors(to.stars, from?.stars ?? null, pair.stars, 3, fraction, out.stars);
  lerpVectors(to.segments, from?.segments ?? null, pair.segments, 6, fraction, out.segments);
  lerpVectors(to.labels, from?.labels ?? null, pair.labels, 3, fraction, out.labels);
  lerpVectors(to.bodies, from?.bodies ?? null, pair.bodies, 3, fraction, out.bodies);
}
