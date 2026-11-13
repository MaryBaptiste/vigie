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

/** Version courte du titre de la carte : `48.86N 2.35E`. */
export function shortLatLon(latDeg: number, lonDeg: number): string {
  const lat = `${Math.abs(latDeg).toFixed(2)}${latDeg < 0 ? "S" : "N"}`;
  const lon = `${Math.abs(lonDeg).toFixed(2)}${lonDeg < 0 ? "W" : "E"}`;
  return `${lat} ${lon}`;
}

export function integer(value: number, width: number): string {
  return Math.trunc(value).toString().padStart(width, " ");
}

/** `2026-10-30T12:34:56.789Z` → `12:34:56`. */
export function utcTime(iso: string): string {
  return new Date(iso).toISOString().slice(11, 19);
}

const pad2 = (n: number): string => n.toString().padStart(2, "0");

/** Heures décimales → `hh:mm:ss`. */
export function hoursToClock(hours: number): string {
  const total = Math.floor((((hours % 24) + 24) % 24) * 3600);
  return `${pad2(Math.floor(total / 3600))}:${pad2(Math.floor(total / 60) % 60)}:${pad2(total % 60)}`;
}

/** Heure UTC `hh:mm:ss`. */
export function utcClock(ms: number): string {
  return new Date(ms).toISOString().slice(11, 19);
}

/** Heure locale de la machine `hh:mm:ss`. */
export function localClock(ms: number): string {
  const d = new Date(ms);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`;
}

/** Heure locale de la machine `hh:mm`, tiret si absente. */
export function localHourMinute(iso: string | null): string {
  if (iso === null) return "--:--";
  const d = new Date(iso);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

/** Date et heure locales `2026-10-30 21:44`. */
export function localDateTime(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

/** Valeur d'un `<input type="datetime-local">` (heure locale), ex. `2026-10-30T21:44`. */
export function toDateTimeLocalValue(ms: number): string {
  return localDateTime(ms).replace(" ", "T");
}

/** `2026-10-29T13:31:42Z` → `2026-10-29 13:31` (UTC). */
export function utcDateTime(iso: string): string {
  return new Date(iso).toISOString().slice(0, 16).replace("T", " ");
}

/** Ascension droite en `hh h mm m ss s`. */
export function rightAscension(deg: number): string {
  const clock = hoursToClock(deg / 15).split(":");
  return `${clock[0]}h${clock[1]}m${clock[2]}s`;
}

/** Angle sexagésimal signé, ex. `+38°47′01″`. */
export function sexagesimal(deg: number): string {
  const total = Math.round(Math.abs(deg) * 3600);
  const d = Math.floor(total / 3600);
  const m = Math.floor(total / 60) % 60;
  const s = total % 60;
  return `${deg < 0 ? "-" : "+"}${pad2(d)}°${pad2(m)}′${pad2(s)}″`;
}

const CARDINALS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;

/** Point cardinal le plus proche d'un azimut. */
export function cardinal(azimuthDeg: number): string {
  const index = Math.round((((azimuthDeg % 360) + 360) % 360) / 45) % 8;
  return CARDINALS[index] ?? "N";
}
