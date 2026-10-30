import type { TrajectoryPoint } from "../types/trajectory";

/**
 * Sépare la trajectoire en portion passée et portion future par rapport à `nowMs`.
 * Le premier point futur ferme aussi la portion passée : pas de trou entre les deux.
 */
export function splitTrajectory(
  points: readonly TrajectoryPoint[],
  nowMs: number,
): { past: TrajectoryPoint[]; future: TrajectoryPoint[] } {
  const index = points.findIndex((p) => Date.parse(p.timestamp) > nowMs);
  if (index === -1) return { past: [...points], future: [] };
  return { past: points.slice(0, index + 1), future: points.slice(index) };
}
