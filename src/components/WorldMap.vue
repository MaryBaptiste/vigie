<script setup lang="ts">
import { computed } from "vue";
import coastlines from "../assets/geo/coastlines.json";
import type { SolarState, TerminatorPoint } from "../types/solar";
import type { Telemetry } from "../types/telemetry";
import type { Trajectory } from "../types/trajectory";
import { MAP_HEIGHT, MAP_WIDTH, project, toMapPolylines } from "../utils/mapProjection";
import { splitTrajectory } from "../utils/trajectory";

const props = defineProps<{
  telemetry: Telemetry | null;
  trajectory: Trajectory | null;
  solar: SolarState | null;
  objectName: string | null;
}>();

const GRID_STEP_DEG = 30;
const MARKER_RADIUS = 2;
const CROSSHAIR = 5;
/** Au-delà, l'étiquette de l'objet passe à gauche du marqueur pour rester visible. */
const LABEL_FLIP_X = MAP_WIDTH - 40;

// ── Fond statique (calculé une seule fois) ────────────────────────────
const coastPolylines = coastlines.flatMap((polyline) =>
  toMapPolylines(polyline.map(([lon = 0, lat = 0]) => ({ latitudeDeg: lat, longitudeDeg: lon }))),
);

interface GridLine {
  key: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  major: boolean;
}

const gridLines: GridLine[] = [
  ...Array.from({ length: 360 / GRID_STEP_DEG - 1 }, (_, i) => {
    const lon = -180 + (i + 1) * GRID_STEP_DEG;
    const [x] = project(0, lon);
    return { key: `lon${lon}`, x1: x, y1: 0, x2: x, y2: MAP_HEIGHT, major: lon === 0 };
  }),
  ...Array.from({ length: 180 / GRID_STEP_DEG - 1 }, (_, i) => {
    const lat = -90 + (i + 1) * GRID_STEP_DEG;
    const [, y] = project(lat, 0);
    return { key: `lat${lat}`, x1: 0, y1: y, x2: MAP_WIDTH, y2: y, major: lat === 0 };
  }),
];

const latitudeLabels = Array.from({ length: 180 / GRID_STEP_DEG - 1 }, (_, i) => {
  const lat = 90 - (i + 1) * GRID_STEP_DEG;
  const label = lat === 0 ? "0" : `${Math.abs(lat)}${lat > 0 ? "N" : "S"}`;
  return { key: lat, y: project(lat, 0)[1], label };
});

// ── Trajectoire ───────────────────────────────────────────────────────
const trajectoryLines = computed(() => {
  const points = props.trajectory?.points ?? [];
  const { past, future } = splitTrajectory(points, Date.now());
  return { past: toMapPolylines(past), future: toMapPolylines(future) };
});

// ── Terminateur ───────────────────────────────────────────────────────
interface TerminatorRun {
  sunrise: boolean;
  polylines: string[];
}

/** Regroupe les points consécutifs de même phase (lever / coucher), raccordés entre eux. */
const terminatorRuns = computed<TerminatorRun[]>(() => {
  const points = props.solar?.terminator ?? [];
  const runs: { sunrise: boolean; points: TerminatorPoint[] }[] = [];
  for (const point of points) {
    const last = runs[runs.length - 1];
    if (last && last.sunrise === point.sunrise) {
      last.points.push(point);
    } else {
      last?.points.push(point);
      runs.push({ sunrise: point.sunrise, points: [point] });
    }
  }
  return runs.map((run) => ({ sunrise: run.sunrise, polylines: toMapPolylines(run.points) }));
});

/** Zone de nuit : sous le terminateur, fermée par le pôle non éclairé. */
const nightPath = computed<string | null>(() => {
  const solar = props.solar;
  if (!solar || solar.terminator.length < 2) return null;
  const poleY = solar.subsolarLatitudeDeg >= 0 ? MAP_HEIGHT : 0;
  const curve = solar.terminator
    .map((p) => project(p.latitudeDeg, p.longitudeDeg))
    .map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`)
    .join(" L ");
  return `M ${curve} L ${MAP_WIDTH},${poleY} L 0,${poleY} Z`;
});

const subsolar = computed(() => {
  const solar = props.solar;
  if (!solar) return null;
  const [x, y] = project(solar.subsolarLatitudeDeg, solar.subsolarLongitudeDeg);
  return { x, y };
});

// ── Objet suivi ───────────────────────────────────────────────────────
const marker = computed(() => {
  const t = props.telemetry;
  if (!t) return null;
  const [x, y] = project(t.latitudeDeg, t.longitudeDeg);
  const flip = x > LABEL_FLIP_X;
  return { x, y, labelX: flip ? x - CROSSHAIR - 1 : x + CROSSHAIR + 1, anchor: flip ? "end" : "start" };
});
</script>

<template>
  <div class="world-map">
    <svg
      class="world-map__svg"
      :viewBox="`0 0 ${MAP_WIDTH} ${MAP_HEIGHT}`"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="World map"
    >
      <rect class="world-map__frame" x="0" y="0" :width="MAP_WIDTH" :height="MAP_HEIGHT" />

      <path v-if="nightPath" class="world-map__night" :d="nightPath" />

      <line
        v-for="line in gridLines"
        :key="line.key"
        class="world-map__grid"
        :class="{ 'world-map__grid--major': line.major }"
        :x1="line.x1"
        :y1="line.y1"
        :x2="line.x2"
        :y2="line.y2"
      />
      <text v-for="l in latitudeLabels" :key="l.key" class="world-map__label" x="2" :y="l.y - 1.5">
        {{ l.label }}
      </text>

      <polyline v-for="(points, i) in coastPolylines" :key="`coast${i}`" class="world-map__coast" :points="points" />

      <polyline
        v-for="(run, i) in terminatorRuns.flatMap((r) => r.polylines.map((points) => ({ sunrise: r.sunrise, points })))"
        :key="`term${i}`"
        class="world-map__terminator"
        :class="run.sunrise ? 'world-map__terminator--sunrise' : 'world-map__terminator--sunset'"
        :points="run.points"
      />

      <g v-if="subsolar" class="world-map__sun" :transform="`translate(${subsolar.x} ${subsolar.y})`">
        <circle r="2.2" />
        <line x1="-4.5" y1="0" x2="-3" y2="0" />
        <line x1="3" y1="0" x2="4.5" y2="0" />
        <line x1="0" y1="-4.5" x2="0" y2="-3" />
        <line x1="0" y1="3" x2="0" y2="4.5" />
      </g>

      <polyline v-for="(points, i) in trajectoryLines.past" :key="`past${i}`" class="world-map__track world-map__track--past" :points="points" />
      <polyline v-for="(points, i) in trajectoryLines.future" :key="`future${i}`" class="world-map__track" :points="points" />

      <g v-if="marker" class="world-map__marker">
        <line :x1="marker.x - CROSSHAIR" :y1="marker.y" :x2="marker.x + CROSSHAIR" :y2="marker.y" />
        <line :x1="marker.x" :y1="marker.y - CROSSHAIR" :x2="marker.x" :y2="marker.y + CROSSHAIR" />
        <circle class="world-map__marker-dot" :cx="marker.x" :cy="marker.y" :r="MARKER_RADIUS" />
        <text
          v-if="objectName"
          class="world-map__marker-label"
          :x="marker.labelX"
          :y="marker.y - 2"
          :text-anchor="marker.anchor"
        >
          {{ objectName }}
        </text>
      </g>
    </svg>

    <ul class="world-map__legend">
      <li><span class="swatch swatch--track" />Track</li>
      <li><span class="swatch swatch--past" />Past</li>
      <li><span class="swatch swatch--sunrise" />Sunrise</li>
      <li><span class="swatch swatch--sunset" />Sunset</li>
      <li><span class="swatch swatch--night" />Night</li>
    </ul>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.world-map {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
}

.world-map__svg {
  flex: 1;
  min-height: 0;
  width: 100%;
  font-family: $font-mono;

  // Épaisseurs en pixels écran, indépendantes du zoom du viewBox.
  line,
  polyline,
  rect,
  circle,
  path {
    vector-effect: non-scaling-stroke;
  }
}

.world-map__frame {
  fill: none;
  stroke: $color-dim;
  stroke-width: 1px;
}

.world-map__night {
  fill: $color-faint;
  fill-opacity: 0.35;
  stroke: none;
}

.world-map__grid {
  stroke: $color-faint;
  stroke-width: 1px;
  stroke-dasharray: 2 3;
}

.world-map__grid--major {
  stroke: $color-dim;
  stroke-dasharray: none;
}

.world-map__label {
  fill: $color-dim;
  font-size: 6px;
}

.world-map__coast {
  fill: none;
  stroke: $color-fg;
  stroke-width: 1.2px;
  stroke-linejoin: round;
}

.world-map__terminator {
  fill: none;
  stroke-width: 2px;
}

.world-map__terminator--sunrise {
  stroke: $color-sun;
}

.world-map__terminator--sunset {
  stroke: $color-sun-dim;
  stroke-dasharray: 4 3;
}

.world-map__sun {
  fill: none;
  stroke: $color-sun;
  stroke-width: 1.5px;
}

.world-map__track {
  fill: none;
  stroke: $color-fg;
  stroke-width: 2px;
  stroke-linejoin: round;
}

.world-map__track--past {
  stroke: $color-dim;
  stroke-width: 1.5px;
}

.world-map__marker {
  stroke: $color-alert;
  stroke-width: 1.5px;
  @include blink;
}

.world-map__marker-dot {
  fill: $color-alert;
}

.world-map__marker-label {
  fill: $color-alert;
  stroke: none;
  font-size: 7px;
}

.world-map__legend {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.5em 2ch;
  padding-top: $space-sm;
  color: $color-dim;
  text-shadow: none;
  @include uppercase-label;

  li {
    display: flex;
    align-items: center;
    gap: 1ch;
  }
}

.swatch {
  display: inline-block;
  width: 3ch;
  height: 0;
  border-top: 2px solid;
}

.swatch--track {
  border-color: $color-fg;
}

.swatch--past {
  border-color: $color-dim;
}

.swatch--sunrise {
  border-color: $color-sun;
}

.swatch--sunset {
  border-top-style: dashed;
  border-color: $color-sun-dim;
}

.swatch--night {
  height: 0.8em;
  border: none;
  background: $color-faint;
  opacity: 0.6;
}
</style>
