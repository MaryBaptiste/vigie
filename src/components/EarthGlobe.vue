<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import {
  BufferGeometry,
  Color,
  Float32BufferAttribute,
  Group,
  Line,
  LineBasicMaterial,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  OctahedronGeometry,
  PerspectiveCamera,
  Scene,
  SphereGeometry,
  Vector3,
  WebGLRenderer,
  type Material,
  type Object3D,
} from "three";
import coastlines from "../assets/geo/coastlines.json";
import type { Telemetry } from "../types/telemetry";
import type { Trajectory, TrajectoryPoint } from "../types/trajectory";
import { latLonToVector3 } from "../utils/geo";
import { splitTrajectory } from "../utils/trajectory";

const props = defineProps<{
  telemetry: Telemetry | null;
  trajectory: Trajectory | null;
}>();

const AXIAL_TILT_DEG = 23.4;
/** Rotation décorative : un tour par minute. */
const ROTATION_PERIOD_S = 60;
const GRATICULE_STEP_DEG = 15;
const ARC_SAMPLE_STEP_DEG = 2;
const CAMERA_FOV_DEG = 30;
/** Rayon englobant de la scène (axe polaire compris) et marge autour. */
const SCENE_BOUND = 1.2;
const FRAME_MARGIN = 1.1;
const MAX_FRAME_DELTA_S = 0.1;
const MARKER_SIZE = 0.03;
const MARKER_PULSE_PERIOD_S = 1.6;
const GHOST_OPACITY = 0.35;

const container = ref<HTMLDivElement | null>(null);

// Objets Three.js volontairement hors du système réactif de Vue.
let renderer: WebGLRenderer | null = null;
let scene: Scene | null = null;
let camera: PerspectiveCamera | null = null;
let earth: Group | null = null;
let resizeObserver: ResizeObserver | null = null;
let lastFrameMs: number | null = null;

/**
 * Calque lié au repère terrestre : trajectoire et marqueur de l'objet suivi.
 * Tout y est en coordonnées géodésiques, donc cohérent avec les continents
 * quelle que soit la rotation (décorative) du globe.
 */
let trackingLayer: Group | null = null;
let trajectoryGroup: Group | null = null;
let marker: Group | null = null;
let markerShape: Mesh | null = null;
let nadirLine: Line | null = null;

interface Palette {
  fg: Color;
  dim: Color;
  faint: Color;
  bg: Color;
}
let palette: Palette | null = null;

function cssColor(name: string): Color {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return new Color(value || "#ffffff");
}

function readPalette(): Palette {
  return {
    fg: cssColor("--color-fg"),
    dim: cssColor("--color-dim"),
    faint: cssColor("--color-faint"),
    bg: cssColor("--color-bg"),
  };
}

type LatLon = [lat: number, lon: number];

/** Ajoute un arc échantillonné sous forme de segments (paires de sommets). */
function pushArc(positions: number[], from: number, to: number, at: (t: number) => LatLon): void {
  const a = new Vector3();
  const b = new Vector3();
  for (let t = from; t < to; t += ARC_SAMPLE_STEP_DEG) {
    latLonToVector3(...at(t), 0, a);
    latLonToVector3(...at(Math.min(t + ARC_SAMPLE_STEP_DEG, to)), 0, b);
    positions.push(a.x, a.y, a.z, b.x, b.y, b.z);
  }
}

function toGeometry(positions: number[]): BufferGeometry {
  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new Float32BufferAttribute(positions, 3));
  return geometry;
}

/** Graticule secondaire, et lignes majeures (équateur + méridien de Greenwich). */
function buildGraticule(): { minor: BufferGeometry; major: BufferGeometry } {
  const minor: number[] = [];
  const major: number[] = [];

  for (let lon = -180; lon < 180; lon += GRATICULE_STEP_DEG) {
    pushArc(lon === 0 ? major : minor, -90, 90, (lat) => [lat, lon]);
  }
  for (let lat = -90 + GRATICULE_STEP_DEG; lat < 90; lat += GRATICULE_STEP_DEG) {
    pushArc(lat === 0 ? major : minor, -180, 180, (lon) => [lat, lon]);
  }

  return { minor: toGeometry(minor), major: toGeometry(major) };
}

/** Côtes Natural Earth 1:110m, polylignes [lon, lat] converties en segments. */
function buildCoastlines(): BufferGeometry {
  const positions: number[] = [];
  const a = new Vector3();
  const b = new Vector3();
  for (const polyline of coastlines) {
    for (let i = 0; i + 1 < polyline.length; i++) {
      const [lonA = 0, latA = 0] = polyline[i] ?? [];
      const [lonB = 0, latB = 0] = polyline[i + 1] ?? [];
      latLonToVector3(latA, lonA, 0, a);
      latLonToVector3(latB, lonB, 0, b);
      positions.push(a.x, a.y, a.z, b.x, b.y, b.z);
    }
  }
  return toGeometry(positions);
}

/** Copie sans test de profondeur, dessinée par-dessus : laisse deviner l'hémisphère arrière. */
function ghostOf(geometry: BufferGeometry, color: Color, kind: "segments" | "line"): Line {
  const material = new LineBasicMaterial({
    color,
    transparent: true,
    opacity: GHOST_OPACITY,
    depthTest: false,
  });
  const ghost = kind === "segments" ? new LineSegments(geometry, material) : new Line(geometry, material);
  ghost.renderOrder = 2;
  return ghost;
}

function buildScene(colors: Palette): Scene {
  const root = new Scene();
  const tilt = new Group();
  tilt.rotation.z = (-AXIAL_TILT_DEG * Math.PI) / 180;
  root.add(tilt);

  earth = new Group();
  tilt.add(earth);

  // Sphère opaque légèrement en retrait : masque l'hémisphère arrière.
  const occluder = new Mesh(new SphereGeometry(0.995, 48, 32), new MeshBasicMaterial({ color: colors.bg }));
  occluder.renderOrder = 0;
  earth.add(occluder);

  const { minor, major } = buildGraticule();
  const coast = buildCoastlines();

  const front = [
    new LineSegments(minor, new LineBasicMaterial({ color: colors.faint })),
    new LineSegments(major, new LineBasicMaterial({ color: colors.dim })),
    new LineSegments(coast, new LineBasicMaterial({ color: colors.fg })),
  ];
  front.forEach((line) => (line.renderOrder = 1));
  earth.add(...front, ghostOf(coast, colors.faint, "segments"), ghostOf(major, colors.faint, "segments"));

  trackingLayer = new Group();
  trackingLayer.name = "tracking";
  earth.add(trackingLayer);

  trajectoryGroup = new Group();
  trackingLayer.add(trajectoryGroup);

  marker = new Group();
  marker.visible = false;
  markerShape = new Mesh(
    new OctahedronGeometry(MARKER_SIZE),
    new MeshBasicMaterial({ color: colors.fg, wireframe: true }),
  );
  markerShape.renderOrder = 3;
  nadirLine = new Line(toGeometry([0, 0, 0, 0, 0, 0]), new LineBasicMaterial({ color: colors.dim }));
  marker.add(markerShape);
  trackingLayer.add(marker, nadirLine);

  // Axe polaire (fixe, ne tourne pas avec la Terre).
  const axis = new Line(
    toGeometry([0, -SCENE_BOUND, 0, 0, SCENE_BOUND, 0]),
    new LineBasicMaterial({ color: colors.faint }),
  );
  tilt.add(axis);

  return root;
}

function disposeObject(root: Object3D): void {
  const materials = new Set<Material>();
  const geometries = new Set<BufferGeometry>();
  root.traverse((object) => {
    if (object instanceof Mesh || object instanceof Line) {
      geometries.add(object.geometry);
      const material: Material | Material[] = object.material;
      (Array.isArray(material) ? material : [material]).forEach((m) => materials.add(m));
    }
  });
  geometries.forEach((g) => g.dispose());
  materials.forEach((m) => m.dispose());
}

/** Reconstruit la trajectoire : passé atténué, futur en pleine intensité. */
function updateTrajectory(trajectory: Trajectory | null): void {
  if (!trajectoryGroup || !palette) return;
  const group = trajectoryGroup;
  [...group.children].forEach((child) => {
    group.remove(child);
    disposeObject(child);
  });
  if (!trajectory || trajectory.points.length < 2) return;

  const v = new Vector3();
  const toPositions = (points: readonly TrajectoryPoint[]): number[] =>
    points.flatMap((p) => latLonToVector3(p.latitudeDeg, p.longitudeDeg, p.altitudeKm, v).toArray());
  const split = splitTrajectory(trajectory.points, Date.now());
  const past = toPositions(split.past);
  const future = toPositions(split.future);
  const all = toPositions(trajectory.points);

  const lines: Line[] = [];
  if (past.length >= 6) lines.push(new Line(toGeometry(past), new LineBasicMaterial({ color: palette.dim })));
  if (future.length >= 6) lines.push(new Line(toGeometry(future), new LineBasicMaterial({ color: palette.fg })));
  lines.forEach((line) => (line.renderOrder = 1));
  group.add(...lines, ghostOf(toGeometry(all), palette.faint, "line"));
}

/** Place le marqueur à la position courante, relié à la surface par sa ligne de nadir. */
function updateMarker(telemetry: Telemetry | null): void {
  if (!marker || !nadirLine) return;
  if (!telemetry) {
    marker.visible = false;
    nadirLine.visible = false;
    return;
  }
  const { latitudeDeg, longitudeDeg, altitudeKm } = telemetry;
  const position = latLonToVector3(latitudeDeg, longitudeDeg, altitudeKm);
  const ground = latLonToVector3(latitudeDeg, longitudeDeg, 0);
  marker.position.copy(position);
  const attribute = nadirLine.geometry.getAttribute("position");
  attribute.setXYZ(0, ground.x, ground.y, ground.z);
  attribute.setXYZ(1, position.x, position.y, position.z);
  attribute.needsUpdate = true;
  nadirLine.geometry.computeBoundingSphere();
  marker.visible = true;
  nadirLine.visible = true;
}

/** Ajuste la distance caméra pour que le globe tienne dans le cadre, quel que soit le ratio. */
function resize(width: number, height: number): void {
  if (!renderer || !camera || width === 0 || height === 0) return;
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  const halfVertical = (CAMERA_FOV_DEG * Math.PI) / 360;
  const halfHorizontal = Math.atan(Math.tan(halfVertical) * camera.aspect);
  const half = Math.min(halfVertical, halfHorizontal);
  camera.position.set(0, 0, (SCENE_BOUND * FRAME_MARGIN) / Math.sin(half));
  camera.lookAt(0, 0, 0);
  camera.updateProjectionMatrix();
}

function animate(timeMs: number): void {
  if (!renderer || !scene || !camera || !earth) return;
  const deltaS = lastFrameMs === null ? 0 : Math.min((timeMs - lastFrameMs) / 1000, MAX_FRAME_DELTA_S);
  lastFrameMs = timeMs;
  earth.rotation.y += (deltaS * 2 * Math.PI) / ROTATION_PERIOD_S;
  if (markerShape) {
    const phase = ((timeMs / 1000) % MARKER_PULSE_PERIOD_S) / MARKER_PULSE_PERIOD_S;
    markerShape.scale.setScalar(1 + 0.35 * Math.sin(phase * 2 * Math.PI));
    markerShape.rotation.y = phase * 2 * Math.PI;
  }
  renderer.render(scene, camera);
}

watch(() => props.trajectory, updateTrajectory);
watch(() => props.telemetry, updateMarker);

onMounted(() => {
  const host = container.value;
  if (!host) return;

  renderer = new WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  host.appendChild(renderer.domElement);

  palette = readPalette();
  scene = buildScene(palette);
  camera = new PerspectiveCamera(CAMERA_FOV_DEG, 1, 0.1, 100);
  resize(host.clientWidth, host.clientHeight);
  updateTrajectory(props.trajectory);
  updateMarker(props.telemetry);

  resizeObserver = new ResizeObserver(([entry]) => {
    if (entry) resize(entry.contentRect.width, entry.contentRect.height);
  });
  resizeObserver.observe(host);

  renderer.setAnimationLoop(animate);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  renderer?.setAnimationLoop(null);
  if (scene) disposeObject(scene);
  renderer?.dispose();
  renderer?.domElement.remove();

  renderer = null;
  scene = null;
  camera = null;
  earth = null;
  trackingLayer = null;
  trajectoryGroup = null;
  marker = null;
  markerShape = null;
  nadirLine = null;
  palette = null;
  resizeObserver = null;
  lastFrameMs = null;
});
</script>

<template>
  <div ref="container" class="earth-globe" />
</template>

<style scoped lang="scss">
.earth-globe {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;

  :deep(canvas) {
    display: block;
    width: 100%;
    height: 100%;
  }
}
</style>
