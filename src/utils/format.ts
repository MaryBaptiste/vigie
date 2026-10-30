/** Formatage à largeur fixe : les valeurs ne « sautent » pas d'une seconde à l'autre. */

export function fixed(value: number, decimals: number, width: number): string {
  return value.toFixed(decimals).padStart(width, " ");
}

/** Nombre signé (+/-) à largeur fixe, ex. ` +51.6234`. */
export function signed(value: number, decimals: number, width: number): string {
  const sign = value < 0 ? "-" : "+";
  return (sign + Math.abs(value).toFixed(decimals)).padStart(width, " ");
}

/** Angle avec hémisphère, ex. `14.2°S`, `043.9°E`. */
export function hemisphere(value: number, positive: string, negative: string, decimals: number, intDigits: number): string {
  const [int = "0", frac] = Math.abs(value).toFixed(decimals).split(".");
  const body = frac === undefined ? int.padStart(intDigits, "0") : `${int.padStart(intDigits, "0")}.${frac}`;
  return `${body}°${value < 0 ? negative : positive}`;
}

export function latLon(latDeg: number, lonDeg: number, decimals = 1): string {
  return `${hemisphere(latDeg, "N", "S", decimals, 2)} ${hemisphere(lonDeg, "E", "W", decimals, 3)}`;
}

export function integer(value: number, width: number): string {
  return Math.trunc(value).toString().padStart(width, " ");
}

/** `2026-10-30T12:34:56.789Z` → `12:34:56`. */
export function utcTime(iso: string): string {
  return new Date(iso).toISOString().slice(11, 19);
}

/** `2026-10-29T13:31:42Z` → `2026-10-29 13:31`. */
export function utcDateTime(iso: string): string {
  return new Date(iso).toISOString().slice(0, 16).replace("T", " ");
}
