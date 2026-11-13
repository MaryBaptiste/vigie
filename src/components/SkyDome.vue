<script setup lang="ts">
import { onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import type { SkySource } from "../composables/useSky";
import type { SkyBody, SkyState } from "../types/sky";
import type { SkyPick } from "../types/skyPick";
import { bodySymbol, bodyTypeLabel, shortBodyName } from "../utils/bodies";
import { fixed } from "../utils/format";
import { mixColors, readPalette, type PhosphorPalette } from "../utils/palette";
import { altitudeRadius, enuToHorizontal, horizonCrossing, stereographic } from "../utils/skyMath";
import { createBuffers, type PreparedFrame } from "../utils/skyInterpolation";

const props = defineProps<{
  source: SkySource;
  showConstellations: boolean;
  showNames: boolean;
  showGrid: boolean;
  /** Objet à centrer et mettre en surbrillance (depuis la page Objets). */
  focusId: string | null;
  /** Hémisphère sud : la Lune croissante est éclairée à gauche. */
  southern: boolean;
}>();

const selected = defineModel<SkyPick | null>("selected", { default: null });

const RAD = Math.PI / 180;
const CARDINALS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"] as const;
const BRIGHT_STAR_LIMIT = 1.5;
const MAX_ZOOM = 8;
const FOCUS_ZOOM = 2.5;
/** Pas de subdivision des segments, pour suivre la courbure de la projection. */
const SEGMENT_STEP_RAD = 6 * RAD;
const HOVER_RADIUS_STAR_PX = 8;
const HOVER_RADIUS_BODY_PX = 14;
const LIVE_UPDATE_MS = 250;
const DRAG_THRESHOLD_PX = 4;

const host = ref<HTMLDivElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const hover = shallowRef<{ pick: SkyPick; x: number; y: number } | null>(null);

// Rendu impératif, hors du système réactif.
let ctx: CanvasRenderingContext2D | null = null;
let palette: PhosphorPalette | null = null;
let starColors: [string, string, string] = ["", "", ""];
let width = 0;
let height = 0;
let dpr = 1;
let fontPx = 16;
let radius = 1;
let rafId = 0;
let resizeObserver: ResizeObserver | null = null;
let lastLiveUpdate = 0;

const buffers = createBuffers();
let frame: PreparedFrame | null = null;
const view = { zoom: 1, cx: 0, cy: 0 };
const target = { zoom: 1, cx: 0, cy: 0 };
let mouse: { x: number; y: number } | null = null;
let drag: { x: number; y: number; cx: number; cy: number; moved: boolean } | null = null;
let pendingFocus: string | null = props.focusId;

watch(
  () => props.focusId,
  (id) => (pendingFocus = id),
);

// ── Géométrie ─────────────────────────────────────────────────────────

/** Coordonnées de la carte (cercle unité) → pixels du canvas. */
function toScreen(xd: number, yd: number): [number, number] {
  const scale = radius * view.zoom;
  return [width / 2 + (xd - view.cx) * scale, height / 2 + (yd - view.cy) * scale];
}

function project(e: number, n: number, u: number): [number, number] {
  const [xd, yd] = stereographic(e, n, u);
  return toScreen(xd, yd);
}

function vectorAt(buffer: Float32Array, offset: number): [number, number, number] {
  return [buffer[offset] ?? 0, buffer[offset + 1] ?? 0, buffer[offset + 2] ?? -1];
}

function clampCenter(): void {
  const limit = Math.max(0, 1 - 1 / target.zoom);
  const distance = Math.hypot(target.cx, target.cy);
  if (distance > limit) {
    target.cx *= limit / distance;
    target.cy *= limit / distance;
  }
}

function resetView(): void {
  target.zoom = 1;
  target.cx = 0;
  target.cy = 0;
}

defineExpose({ resetView });

// ── Dessin ────────────────────────────────────────────────────────────

function font(scale: number): string {
  return `${Math.round(fontPx * scale * dpr)}px VT323, Consolas, monospace`;
}

function skyBackground(state: SkyState, colors: PhosphorPalette): string {
  switch (state) {
    case "NIGHT":
      return colors.skyNight;
    case "ASTRONOMICAL TWILIGHT":
      return mixColors(colors.skyNight, colors.skyTwilight, 0.5);
    case "NAUTICAL TWILIGHT":
      return colors.skyTwilight;
    case "CIVIL TWILIGHT":
      return mixColors(colors.skyTwilight, colors.skyDay, 0.5);
    case "DAY":
      return colors.skyDay;
  }
}

/** Les étoiles pâlissent quand le ciel s'éclaire (sans disparaître : c'est une carte). */
function starDimming(state: SkyState): number {
  if (state === "DAY") return 0.35;
  if (state === "CIVIL TWILIGHT") return 0.6;
  return 1;
}

function drawGrid(c: CanvasRenderingContext2D, colors: PhosphorPalette, hx: number, hy: number, hr: number): void {
  c.save();
  c.strokeStyle = colors.faint;
  c.lineWidth = dpr;
  c.setLineDash([2 * dpr, 4 * dpr]);
  c.beginPath();
  for (const altitude of [30, 60]) {
    const r = altitudeRadius(altitude) * hr;
    c.moveTo(hx + r, hy);
    c.arc(hx, hy, r, 0, 2 * Math.PI);
  }
  for (let az = 0; az < 360; az += 45) {
    c.moveTo(hx, hy);
    c.lineTo(hx - Math.sin(az * RAD) * hr, hy - Math.cos(az * RAD) * hr);
  }
  c.stroke();
  c.setLineDash([]);
  c.fillStyle = colors.faint;
  c.font = font(0.7);
  c.textAlign = "left";
  for (const altitude of [30, 60]) {
    c.fillText(`${altitude}°`, hx + 3 * dpr, hy - altitudeRadius(altitude) * hr - 2 * dpr);
  }
  c.restore();
}

function drawSegments(c: CanvasRenderingContext2D, colors: PhosphorPalette): void {
  const segments = buffers.segments;
  c.beginPath();
  for (let o = 0; o + 5 < segments.length; o += 6) {
    let a = vectorAt(segments, o);
    let b = vectorAt(segments, o + 3);
    if (a[2] < 0 && b[2] < 0) continue;
    // Découpe à l'horizon des segments qui le traversent.
    if (a[2] < 0) a = horizonCrossing(b, a);
    if (b[2] < 0) b = horizonCrossing(a, b);
    const dot = Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1] + a[2] * b[2]));
    const pieces = Math.max(1, Math.ceil(Math.acos(dot) / SEGMENT_STEP_RAD));
    for (let k = 0; k <= pieces; k++) {
      const t = k / pieces;
      const e = a[0] + (b[0] - a[0]) * t;
      const n = a[1] + (b[1] - a[1]) * t;
      const u = a[2] + (b[2] - a[2]) * t;
      const norm = Math.hypot(e, n, u) || 1;
      const [x, y] = project(e / norm, n / norm, u / norm);
      if (k === 0) c.moveTo(x, y);
      else c.lineTo(x, y);
    }
  }
  c.strokeStyle = colors.dim;
  c.lineWidth = dpr;
  c.stroke();
}

function drawConstellationNames(c: CanvasRenderingContext2D, colors: PhosphorPalette, current: PreparedFrame): void {
  c.fillStyle = colors.dim;
  c.font = font(0.8);
  c.textAlign = "center";
  c.textBaseline = "middle";
  current.sky.constellations.forEach((constellation, i) => {
    const [e, n, u] = vectorAt(buffers.labels, i * 3);
    if (Number.isNaN(e) || u < 0.03) return;
    const [x, y] = project(e, n, u);
    c.fillText(constellation.latinName.toUpperCase(), x, y);
  });
}

function starRadius(magnitude: number): number {
  const zoomBoost = 1 + 0.25 * Math.log2(view.zoom);
  return Math.max(0.55, Math.min(3.4, 2.5 - 0.4 * magnitude)) * dpr * zoomBoost;
}

function drawStars(c: CanvasRenderingContext2D, colors: PhosphorPalette, current: PreparedFrame): void {
  const dimming = starDimming(current.sky.skyState);
  // Regroupement par opacité (6 niveaux) et teinte (neutre, chaude, froide) : un fill par groupe.
  const paths = Array.from({ length: 18 }, () => new Path2D());
  const bright: { x: number; y: number; r: number }[] = [];
  current.sky.stars.forEach((star, i) => {
    const [e, n, u] = vectorAt(buffers.stars, i * 3);
    if (u < 0) return;
    const [x, y] = project(e, n, u);
    if (x < 0 || y < 0 || x > width || y > height) return;
    const r = starRadius(star.magnitude);
    if (star.magnitude < BRIGHT_STAR_LIMIT) {
      bright.push({ x, y, r });
      return;
    }
    const level = Math.min(5, Math.max(0, Math.floor((star.magnitude - BRIGHT_STAR_LIMIT) / 0.8)));
    const bv = star.colorIndex;
    const tint = bv === null ? 0 : bv < 0 ? 2 : bv > 1.2 ? 1 : 0;
    const path = paths[tint * 6 + level];
    path?.moveTo(x + r, y);
    path?.arc(x, y, r, 0, 2 * Math.PI);
  });
  paths.forEach((path, index) => {
    const level = index % 6;
    c.globalAlpha = Math.max(0.28, 0.95 - level * 0.13) * dimming;
    c.fillStyle = starColors[Math.floor(index / 6)] ?? colors.fg;
    c.fill(path);
  });
  // Étoiles brillantes : léger halo.
  c.globalAlpha = dimming;
  c.fillStyle = colors.fg;
  c.shadowColor = colors.glow;
  c.shadowBlur = 8 * dpr;
  for (const { x, y, r } of bright) {
    c.beginPath();
    c.arc(x, y, r, 0, 2 * Math.PI);
    c.fill();
  }
  c.shadowBlur = 0;
  c.globalAlpha = 1;
}

function drawStarNames(c: CanvasRenderingContext2D, colors: PhosphorPalette, current: PreparedFrame): void {
  c.fillStyle = colors.dim;
  c.font = font(0.7);
  c.textAlign = "left";
  c.textBaseline = "bottom";
  current.sky.stars.forEach((star, i) => {
    if (!star.name || star.magnitude >= BRIGHT_STAR_LIMIT) return;
    const [e, n, u] = vectorAt(buffers.stars, i * 3);
    if (u < 0) return;
    const [x, y] = project(e, n, u);
    c.fillText(star.name, x + 4 * dpr, y - 2 * dpr);
  });
}

/** Disque lunaire : demi-disque éclairé + demi-ellipse du terminateur. */
function drawMoon(c: CanvasRenderingContext2D, colors: PhosphorPalette, x: number, y: number, body: SkyBody): void {
  const r = 7 * dpr;
  const k = body.illumination ?? 1;
  const litOnRight = (body.waxing ?? true) !== props.southern;
  c.beginPath();
  c.arc(x, y, r, 0, 2 * Math.PI);
  c.fillStyle = colors.faint;
  c.fill();
  c.beginPath();
  c.arc(x, y, r, -Math.PI / 2, Math.PI / 2, !litOnRight);
  const bulgeTowardDark = k > 0.5;
  c.ellipse(x, y, r * Math.abs(1 - 2 * k), r, 0, Math.PI / 2, -Math.PI / 2, litOnRight !== bulgeTowardDark);
  c.fillStyle = colors.fg;
  c.shadowColor = colors.glow;
  c.shadowBlur = 10 * dpr;
  c.fill();
  c.shadowBlur = 0;
}

function drawBodies(c: CanvasRenderingContext2D, colors: PhosphorPalette, current: PreparedFrame): void {
  c.textBaseline = "middle";
  current.sky.bodies.forEach((body, i) => {
    const [e, n, u] = vectorAt(buffers.bodies, i * 3);
    if (u < -0.005) return;
    const [x, y] = project(e, n, u);
    c.shadowColor = colors.glow;
    if (body.bodyType === "SUN") {
      c.strokeStyle = colors.sun;
      c.fillStyle = colors.sun;
      c.lineWidth = 1.5 * dpr;
      c.shadowBlur = 12 * dpr;
      c.beginPath();
      c.arc(x, y, 7 * dpr, 0, 2 * Math.PI);
      c.stroke();
      c.beginPath();
      c.arc(x, y, 1.5 * dpr, 0, 2 * Math.PI);
      c.fill();
      c.shadowBlur = 0;
    } else if (body.bodyType === "MOON") {
      drawMoon(c, colors, x, y, body);
    } else {
      c.fillStyle = body.bodyType === "PLANET" ? colors.alert : colors.fg;
      c.font = font(1.2);
      c.textAlign = "center";
      c.shadowBlur = 6 * dpr;
      c.fillText(bodySymbol(body.id, body.bodyType), x, y);
      c.shadowBlur = 0;
    }
    if (props.showNames) {
      c.fillStyle = body.bodyType === "SUN" ? colors.sun : colors.fg;
      c.font = font(0.8);
      c.textAlign = "left";
      c.fillText(shortBodyName(body.name).toUpperCase(), x + 11 * dpr, y + 1 * dpr);
    }
  });
}

function drawHorizon(c: CanvasRenderingContext2D, colors: PhosphorPalette, hx: number, hy: number, hr: number): void {
  c.strokeStyle = colors.fg;
  c.lineWidth = 2 * dpr;
  c.shadowColor = colors.glow;
  c.shadowBlur = 6 * dpr;
  c.beginPath();
  c.arc(hx, hy, hr, 0, 2 * Math.PI);
  c.stroke();
  c.shadowBlur = 0;

  c.lineWidth = dpr;
  c.beginPath();
  for (let az = 0; az < 360; az += 15) {
    const length = (az % 45 === 0 ? 9 : 4) * dpr;
    const dx = -Math.sin(az * RAD);
    const dy = -Math.cos(az * RAD);
    c.moveTo(hx + dx * hr, hy + dy * hr);
    c.lineTo(hx + dx * (hr - length), hy + dy * (hr - length));
  }
  c.stroke();

  c.fillStyle = colors.fg;
  c.textAlign = "center";
  c.textBaseline = "middle";
  CARDINALS.forEach((label, i) => {
    const az = i * 45 * RAD;
    const offset = hr + fontPx * (label.length > 1 ? 1.0 : 0.8) * dpr;
    c.font = font(label.length > 1 ? 0.85 : 1.1);
    c.fillText(label, hx - Math.sin(az) * offset, hy - Math.cos(az) * offset);
  });
}

/** Réticule clignotant autour de l'objet épinglé. */
function drawReticle(c: CanvasRenderingContext2D, colors: PhosphorPalette, x: number, y: number, timeMs: number): void {
  if (Math.floor(timeMs / 500) % 2 === 1) return;
  const s = 11 * dpr;
  const l = 5 * dpr;
  c.strokeStyle = colors.alert;
  c.lineWidth = 1.5 * dpr;
  c.beginPath();
  for (const [sx, sy] of [
    [-1, -1],
    [1, -1],
    [1, 1],
    [-1, 1],
  ] as const) {
    c.moveTo(x + sx * s, y + sy * (s - l));
    c.lineTo(x + sx * s, y + sy * s);
    c.lineTo(x + sx * (s - l), y + sy * s);
  }
  c.stroke();
}

// ── Sélection ─────────────────────────────────────────────────────────

function pickBody(current: PreparedFrame, index: number): SkyPick | null {
  const body = current.sky.bodies[index];
  if (!body) return null;
  const [e, n, u] = vectorAt(buffers.bodies, index * 3);
  const position = enuToHorizontal(e, n, u);
  return {
    kind: "body",
    id: body.id,
    name: body.name,
    subtitle: bodyTypeLabel(body.bodyType),
    magnitude: body.magnitude,
    ...position,
    illumination: body.illumination,
  };
}

function pickStar(current: PreparedFrame, index: number): SkyPick | null {
  const star = current.sky.stars[index];
  if (!star) return null;
  const [e, n, u] = vectorAt(buffers.stars, index * 3);
  return {
    kind: "star",
    id: String(star.id),
    name: star.name ?? `HR ${star.id}`,
    subtitle: `STAR · HR ${star.id}`,
    magnitude: star.magnitude,
    ...enuToHorizontal(e, n, u),
    illumination: null,
  };
}

/** Objet le plus proche d'un point du canvas (pixels physiques). */
/** Indice et distance (pixels) de l'élément le plus proche dans un rayon donné. */
function nearest(buffer: Float32Array, count: number, px: number, py: number, radiusPx: number) {
  let best: { index: number; distance: number } | null = null;
  for (let i = 0; i < count; i++) {
    const [e, n, u] = vectorAt(buffer, i * 3);
    if (u < -0.005) continue;
    const [x, y] = project(e, n, u);
    const distance = Math.hypot(x - px, y - py);
    if (distance <= radiusPx * dpr && (!best || distance < best.distance)) best = { index: i, distance };
  }
  return best;
}

/** Objet le plus proche d'un point du canvas (pixels physiques). */
function pickAt(current: PreparedFrame, px: number, py: number): SkyPick | null {
  const body = nearest(buffers.bodies, current.sky.bodies.length, px, py, HOVER_RADIUS_BODY_PX);
  const star = nearest(buffers.stars, current.sky.stars.length, px, py, HOVER_RADIUS_STAR_PX);
  // Les objets du système solaire passent devant les étoiles voisines.
  if (body && (!star || body.distance - 6 * dpr <= star.distance)) return pickBody(current, body.index);
  return star ? pickStar(current, star.index) : null;
}

function screenOf(current: PreparedFrame, pick: SkyPick): [number, number] | null {
  const list = pick.kind === "body" ? current.sky.bodies : current.sky.stars;
  const index = list.findIndex((item) => String(item.id) === pick.id);
  if (index < 0) return null;
  const [e, n, u] = vectorAt(pick.kind === "body" ? buffers.bodies : buffers.stars, index * 3);
  return u < -0.005 ? null : project(e, n, u);
}

function refreshPick(current: PreparedFrame, pick: SkyPick): SkyPick {
  const list = pick.kind === "body" ? current.sky.bodies : current.sky.stars;
  const index = list.findIndex((item) => String(item.id) === pick.id);
  if (index < 0) return pick;
  return (pick.kind === "body" ? pickBody(current, index) : pickStar(current, index)) ?? pick;
}

function applyPendingFocus(current: PreparedFrame): void {
  if (!pendingFocus) return;
  const index = current.sky.bodies.findIndex((b) => b.id === pendingFocus);
  pendingFocus = null;
  const pick = pickBody(current, index);
  if (!pick) return;
  selected.value = pick;
  const [e, n, u] = vectorAt(buffers.bodies, index * 3);
  if (u < 0) return;
  const [xd, yd] = stereographic(e, n, u);
  target.zoom = FOCUS_ZOOM;
  target.cx = xd;
  target.cy = yd;
  clampCenter();
}

// ── Boucle d'animation ────────────────────────────────────────────────

function render(timeMs: number): void {
  rafId = requestAnimationFrame(render);
  const c = ctx;
  const colors = palette;
  if (!c || !colors || width === 0) return;

  frame = props.source.frameAt(props.source.simNowMs(), buffers) ?? frame;
  view.zoom += (target.zoom - view.zoom) * 0.2;
  view.cx += (target.cx - view.cx) * 0.2;
  view.cy += (target.cy - view.cy) * 0.2;

  c.setTransform(1, 0, 0, 1, 0, 0);
  c.fillStyle = colors.bg;
  c.fillRect(0, 0, width, height);
  const current = frame;
  if (!current) {
    c.fillStyle = colors.dim;
    c.font = font(1);
    c.textAlign = "center";
    c.fillText("ACQUIRING SKY…", width / 2, height / 2);
    return;
  }
  applyPendingFocus(current);

  const [hx, hy] = toScreen(0, 0);
  const hr = radius * view.zoom;
  c.save();
  c.beginPath();
  c.arc(hx, hy, hr, 0, 2 * Math.PI);
  c.fillStyle = skyBackground(current.sky.skyState, colors);
  c.fill();
  c.clip();
  if (props.showGrid) drawGrid(c, colors, hx, hy, hr);
  if (props.showConstellations) {
    drawSegments(c, colors);
    if (props.showNames) drawConstellationNames(c, colors, current);
  }
  drawStars(c, colors, current);
  if (props.showNames) drawStarNames(c, colors, current);
  drawBodies(c, colors, current);
  c.restore();
  drawHorizon(c, colors, hx, hy, hr);

  if (selected.value) {
    const position = screenOf(current, selected.value);
    if (position) drawReticle(c, colors, position[0], position[1], timeMs);
  }

  if (timeMs - lastLiveUpdate > LIVE_UPDATE_MS) {
    lastLiveUpdate = timeMs;
    if (selected.value) selected.value = refreshPick(current, selected.value);
    if (mouse && !drag?.moved) {
      const pick = pickAt(current, mouse.x * dpr, mouse.y * dpr);
      hover.value = pick ? { pick, x: mouse.x, y: mouse.y } : null;
    }
  }
}

// ── Souris ────────────────────────────────────────────────────────────

function localPoint(event: MouseEvent): { x: number; y: number } {
  const rect = canvas.value?.getBoundingClientRect();
  return { x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) };
}

function onMouseDown(event: MouseEvent): void {
  const point = localPoint(event);
  drag = { ...point, cx: target.cx, cy: target.cy, moved: false };
}

function onMouseMove(event: MouseEvent): void {
  mouse = localPoint(event);
  if (!drag) {
    if (frame) {
      const pick = pickAt(frame, mouse.x * dpr, mouse.y * dpr);
      hover.value = pick ? { pick, x: mouse.x, y: mouse.y } : null;
    }
    return;
  }
  const dx = mouse.x - drag.x;
  const dy = mouse.y - drag.y;
  if (Math.hypot(dx, dy) > DRAG_THRESHOLD_PX) drag.moved = true;
  if (drag.moved && target.zoom > 1) {
    const scale = (radius * view.zoom) / dpr;
    target.cx = drag.cx - dx / scale;
    target.cy = drag.cy - dy / scale;
    clampCenter();
    view.cx = target.cx;
    view.cy = target.cy;
    hover.value = null;
  }
}

function onMouseUp(event: MouseEvent): void {
  if (drag && !drag.moved && frame) {
    const point = localPoint(event);
    selected.value = pickAt(frame, point.x * dpr, point.y * dpr);
  }
  drag = null;
}

function onMouseLeave(): void {
  mouse = null;
  drag = null;
  hover.value = null;
}

/** Zoom à la molette, centré sur le curseur. */
function onWheel(event: WheelEvent): void {
  const point = localPoint(event);
  const scale = radius * target.zoom;
  const xd = target.cx + (point.x * dpr - width / 2) / scale;
  const yd = target.cy + (point.y * dpr - height / 2) / scale;
  target.zoom = Math.max(1, Math.min(MAX_ZOOM, target.zoom * Math.exp(-event.deltaY * 0.0015)));
  const newScale = radius * target.zoom;
  target.cx = xd - (point.x * dpr - width / 2) / newScale;
  target.cy = yd - (point.y * dpr - height / 2) / newScale;
  clampCenter();
}

// ── Cycle de vie ──────────────────────────────────────────────────────

function resize(cssWidth: number, cssHeight: number): void {
  const element = canvas.value;
  if (!element || !host.value) return;
  dpr = Math.min(window.devicePixelRatio || 1, 2);
  width = Math.round(cssWidth * dpr);
  height = Math.round(cssHeight * dpr);
  element.width = width;
  element.height = height;
  fontPx = Number.parseFloat(getComputedStyle(host.value).fontSize) || 16;
  // Marge pour les points cardinaux autour de l'horizon.
  radius = Math.max(10, Math.min(width, height) / 2 - fontPx * 1.9 * dpr);
}

onMounted(() => {
  const element = canvas.value;
  if (!element || !host.value) return;
  ctx = element.getContext("2d");
  palette = readPalette();
  starColors = [palette.fg, mixColors(palette.fg, palette.danger, 0.3), mixColors(palette.fg, palette.sun, 0.3)];
  resize(host.value.clientWidth, host.value.clientHeight);
  resizeObserver = new ResizeObserver(([entry]) => {
    if (entry) resize(entry.contentRect.width, entry.contentRect.height);
  });
  resizeObserver.observe(host.value);
  rafId = requestAnimationFrame(render);
});

onUnmounted(() => {
  cancelAnimationFrame(rafId);
  resizeObserver?.disconnect();
  resizeObserver = null;
  ctx = null;
});
</script>

<template>
  <div ref="host" class="sky-dome">
    <canvas
      ref="canvas"
      class="sky-dome__canvas"
      :class="{ 'sky-dome__canvas--pointer': hover }"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseLeave"
      @wheel.prevent="onWheel"
    />
    <div v-if="hover" class="sky-dome__tooltip" :style="{ left: `${hover.x + 16}px`, top: `${hover.y + 16}px` }">
      <div class="sky-dome__tooltip-name">{{ hover.pick.name }}</div>
      <div class="sky-dome__tooltip-row">MAG {{ fixed(hover.pick.magnitude, 1, 5) }}</div>
      <div class="sky-dome__tooltip-row">AZ  {{ fixed(hover.pick.azimuthDeg, 1, 5) }}°</div>
      <div class="sky-dome__tooltip-row">ALT {{ fixed(hover.pick.altitudeDeg, 1, 5) }}°</div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.sky-dome {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.sky-dome__canvas {
  display: block;
  width: 100%;
  height: 100%;
  cursor: crosshair;
}

.sky-dome__canvas--pointer {
  cursor: pointer;
}

.sky-dome__tooltip {
  position: absolute;
  z-index: 2;
  padding: $space-xs 1ch;
  border: 1px solid $color-dim;
  background: $color-bg;
  pointer-events: none;
  box-shadow: 0 0 6px $color-glow;
  white-space: pre;
}

.sky-dome__tooltip-name {
  @include uppercase-label;
  color: $color-alert;
}

.sky-dome__tooltip-row {
  color: $color-fg;
}
</style>
