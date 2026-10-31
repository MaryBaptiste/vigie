/** Rendus ASCII : phase de la Lune et courbe de hauteur. */

/**
 * Disque lunaire en caractères. `phaseAngleDeg` : 0 = pleine lune, 180 = nouvelle.
 * Croissante : éclairée à droite (hémisphère nord) ; `mirrored` inverse pour le sud.
 */
export function moonPhaseAscii(phaseAngleDeg: number, waxing: boolean, mirrored: boolean, rows = 9): string[] {
  // Les caractères sont environ deux fois plus hauts que larges.
  const cols = rows * 2 + 1;
  const cosPhase = Math.cos((phaseAngleDeg * Math.PI) / 180);
  const litOnRight = waxing !== mirrored;
  const lines: string[] = [];
  for (let r = 0; r < rows; r++) {
    const y = ((r + 0.5) / rows) * 2 - 1;
    let line = "";
    for (let c = 0; c < cols; c++) {
      const x = ((c + 0.5) / cols) * 2 - 1;
      const half = Math.sqrt(Math.max(0, 1 - y * y));
      if (Math.abs(x) > half) {
        line += " ";
        continue;
      }
      // Terminateur : demi-ellipse de demi-largeur |cos i|.
      const lit = litOnRight ? x > -half * cosPhase : x < half * cosPhase;
      line += lit ? "█" : "·";
    }
    lines.push(line);
  }
  return lines;
}

const LEVELS = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"] as const;

/**
 * Courbe de hauteur (0–90°) sur `rows` lignes : une colonne par échantillon,
 * blocs à huit niveaux par ligne ; `·` sous l'horizon.
 */
export function altitudeChart(altitudes: readonly number[], rows = 6, maxAltitude = 90): string[] {
  const lines: string[] = [];
  for (let r = rows - 1; r >= 0; r--) {
    let line = "";
    for (const altitude of altitudes) {
      if (altitude <= 0) {
        line += r === 0 ? "·" : " ";
        continue;
      }
      const height = (Math.min(altitude, maxAltitude) / maxAltitude) * rows;
      const fill = Math.max(0, Math.min(1, height - r));
      line += LEVELS[Math.round(fill * 8)] ?? " ";
    }
    lines.push(line);
  }
  return lines;
}
