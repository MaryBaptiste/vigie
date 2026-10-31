/** Couleurs de la palette phosphore lues depuis les custom properties CSS, pour Canvas et Three.js. */

export interface PhosphorPalette {
  fg: string;
  dim: string;
  faint: string;
  bg: string;
  glow: string;
  alert: string;
  danger: string;
  sun: string;
  skyNight: string;
  skyTwilight: string;
  skyDay: string;
}

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

export function readPalette(): PhosphorPalette {
  return {
    fg: cssVar("--color-fg", "#ffffff"),
    dim: cssVar("--color-dim", "#888888"),
    faint: cssVar("--color-faint", "#444444"),
    bg: cssVar("--color-bg", "#000000"),
    glow: cssVar("--color-glow", "rgba(255, 255, 255, 0.4)"),
    alert: cssVar("--color-alert", "#ffffff"),
    danger: cssVar("--color-danger", "#ffffff"),
    sun: cssVar("--color-sun", "#ffffff"),
    skyNight: cssVar("--color-sky-night", "#000000"),
    skyTwilight: cssVar("--color-sky-twilight", "#111111"),
    skyDay: cssVar("--color-sky-day", "#222222"),
  };
}

function parseHex(color: string): [number, number, number] | null {
  const match = /^#([0-9a-f]{6})$/i.exec(color);
  if (!match?.[1]) return null;
  const value = Number.parseInt(match[1], 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

/** Mélange linéaire de deux couleurs `#rrggbb` (t = 0 → a, 1 → b). */
export function mixColors(a: string, b: string, t: number): string {
  const ca = parseHex(a);
  const cb = parseHex(b);
  if (!ca || !cb) return a;
  const channel = (i: 0 | 1 | 2) => Math.round(ca[i] + (cb[i] - ca[i]) * t);
  return `rgb(${channel(0)}, ${channel(1)}, ${channel(2)})`;
}
