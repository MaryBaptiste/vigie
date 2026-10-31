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
  PerspectiveCamera,
  Raycaster,
  Scene,
  SphereGeometry,
  Vector2,
  Vector3,
  WebGLRenderer,
  type Material,
  type Object3D,
} from "three";
import coastlines from "../assets/geo/coastlines.json";
import { latLonToVector3, vector3ToLatLon } from "../utils/geo";
import { readPalette } from "../utils/palette";

const props = defineProps<{
  latitudeDeg: number;
  longitudeDeg: number;
}>();

const emit = defineEmits<{ pick: [latitudeDeg: number, longitudeDeg: number] }>();

const GRATICULE_STEP_DEG = 15;
const ARC_SAMPLE_STEP_DEG = 2;
const CAMERA_FOV_DEG = 30;
const SCENE_BOUND = 1.05;
const FRAME_MARGIN = 1.1;
const MIN_ZOOM = 1;
const MAX_ZOOM = 4;
/** Réticule : rayon angulaire (rad) et altitude au-dessus du sol. */
const RETICLE_RADIUS = 0.06;
const RETICLE_LIFT = 1.004;
const GHOST_OPACITY = 0.3;
const DRAG_THRESHOLD_PX = 4;
/** Fraction du chemin restant parcourue à chaque image (recentrage en douceur). */
const EASING = 0.12;
const DEG = Math.PI / 180;

const container = ref<HTMLDivElement | null>(null);

// Objets Three.js volontairement hors du système réactif de Vue.
let renderer: WebGLRenderer | null = null;
let scene: Scene | null = null;
let camera: PerspectiveCamera | null = null;
/** Inclinaison (latitude visée) puis rotation (longitude visée). */
let pitch: Group | null = null;
let earth: Group | null = null;
let occluder: Mesh | null = null;
let reticle: Group | null = null;
let resizeObserver: ResizeObserver | null = null;
let baseDistance = 4;

/** Orientation affichée et orientation visée (degrés), zoom. */
const view = { lat: props.latitudeDeg, lon: props.longitudeDeg, zoom: 1.4 };
const goal = { lat: props.latitudeDeg, lon: props.longitudeDeg, zoom: 1.4 };
let drag: { x: number; y: number; lat: number; lon: number; moved: boolean } | null = null;
const raycaster = new Raycaster();

type LatLon = [lat: number, lon: number];

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

function ghostOf(geometry: BufferGeometry, color: Color): LineSegments {
  const ghost = new LineSegments(
    geometry,
    new LineBasicMaterial({ color, transparent: true, opacity: GHOST_OPACITY, depthTest: false }),
  );
  ghost.renderOrder = 2;
  return ghost;
}

/** Réticule : cercle et croix dans le plan tangent, posé sur +Z puis orienté. */
function buildReticle(color: Color): Group {
  const positions: number[] = [];
  const steps = 32;
  for (let i = 0; i < steps; i++) {
    const a = (i / steps) * 2 * Math.PI;
    const b = ((i + 1) / steps) * 2 * Math.PI;
    positions.push(Math.cos(a) * RETICLE_RADIUS, Math.sin(a) * RETICLE_RADIUS, 0);
    positions.push(Math.cos(b) * RETICLE_RADIUS, Math.sin(b) * RETICLE_RADIUS, 0);
  }
  const arm = RETICLE_RADIUS * 1.8;
  const gap = RETICLE_RADIUS * 0.4;
  positions.push(-arm, 0, 0, -gap, 0, 0, gap, 0, 0, arm, 0, 0, 0, -arm, 0, 0, -gap, 0, 0, gap, 0, 0, arm, 0);
  const lines = new LineSegments(toGeometry(positions), new LineBasicMaterial({ color, depthTest: false }));
  lines.renderOrder = 3;
  const group = new Group();
  group.add(lines);
  return group;
}

function placeReticle(latDeg: number, lonDeg: number): void {
  if (!reticle) return;
  const normal = latLonToVector3(latDeg, lonDeg, 0);
  reticle.position.copy(normal).multiplyScalar(RETICLE_LIFT);
  // Plan du réticule (normale +Z) tangent à la sphère, en repère terrestre.
  reticle.quaternion.setFromUnitVectors(new Vector3(0, 0, 1), normal);
}

function buildScene(): Scene {
  const colors = readPalette();
  const fg = new Color(colors.fg);
  const dim = new Color(colors.dim);
  const faint = new Color(colors.faint);
  const root = new Scene();
  pitch = new Group();
  root.add(pitch);
  earth = new Group();
  pitch.add(earth);

  // Sphère opaque légèrement en retrait : masque l'hémisphère arrière, et cible du raycast.
  occluder = new Mesh(new SphereGeometry(0.995, 48, 32), new MeshBasicMaterial({ color: new Color(colors.bg) }));
  occluder.renderOrder = 0;
  earth.add(occluder);

  const { minor, major } = buildGraticule();
  const coast = buildCoastlines();
  const front = [
    new LineSegments(minor, new LineBasicMaterial({ color: faint })),
    new LineSegments(major, new LineBasicMaterial({ color: dim })),
    new LineSegments(coast, new LineBasicMaterial({ color: fg })),
  ];
  front.forEach((line) => (line.renderOrder = 1));
  earth.add(...front, ghostOf(coast, faint));

  reticle = buildReticle(new Color(colors.alert));
  earth.add(reticle);
  placeReticle(props.latitudeDeg, props.longitudeDeg);
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

function resize(width: number, height: number): void {
  if (!renderer || !camera || width === 0 || height === 0) return;
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  const halfVertical = (CAMERA_FOV_DEG * Math.PI) / 360;
  const half = Math.min(halfVertical, Math.atan(Math.tan(halfVertical) * camera.aspect));
  baseDistance = (SCENE_BOUND * FRAME_MARGIN) / Math.sin(half);
  camera.updateProjectionMatrix();
}

/** Plus court chemin angulaire en longitude. */
function wrapDelta(delta: number): number {
  return ((((delta + 180) % 360) + 360) % 360) - 180;
}

function animate(): void {
  if (!renderer || !scene || !camera || !pitch || !earth) return;
  view.lat += (goal.lat - view.lat) * EASING;
  view.lon += wrapDelta(goal.lon - view.lon) * EASING;
  view.zoom += (goal.zoom - view.zoom) * EASING;
  // Le point (lat, lon) visé fait face à la caméra : rotation en longitude, puis inclinaison.
  earth.rotation.y = -view.lon * DEG;
  pitch.rotation.x = view.lat * DEG;
  // Au-delà du rayon terrestre, la distance se réduit avec le zoom.
  camera.position.set(0, 0, 1 + (baseDistance - 1) / view.zoom);
  camera.lookAt(0, 0, 0);
  renderer.render(scene, camera);
}

// ── Souris ────────────────────────────────────────────────────────────

function onPointerDown(event: PointerEvent): void {
  drag = { x: event.clientX, y: event.clientY, lat: goal.lat, lon: goal.lon, moved: false };
  (event.currentTarget as HTMLElement | null)?.setPointerCapture(event.pointerId);
}

function onPointerMove(event: PointerEvent): void {
  if (!drag || !container.value) return;
  const dx = event.clientX - drag.x;
  const dy = event.clientY - drag.y;
  if (Math.hypot(dx, dy) > DRAG_THRESHOLD_PX) drag.moved = true;
  if (!drag.moved) return;
  // Environ 180° pour la largeur du globe à zoom 1, moins quand on zoome.
  const degreesPerPixel = 180 / container.value.clientHeight / goal.zoom;
  goal.lon = drag.lon - dx * degreesPerPixel;
  goal.lat = Math.max(-89, Math.min(89, drag.lat + dy * degreesPerPixel));
}

function onPointerUp(event: PointerEvent): void {
  const wasClick = drag && !drag.moved;
  drag = null;
  if (!wasClick || !camera || !occluder || !earth || !container.value) return;
  const rect = container.value.getBoundingClientRect();
  const ndc = new Vector2(
    ((event.clientX - rect.left) / rect.width) * 2 - 1,
    -((event.clientY - rect.top) / rect.height) * 2 + 1,
  );
  raycaster.setFromCamera(ndc, camera);
  const hit = raycaster.intersectObject(occluder, false)[0];
  if (!hit) return;
  const [lat, lon] = vector3ToLatLon(earth.worldToLocal(hit.point.clone()));
  emit("pick", lat, lon);
}

function onWheel(event: WheelEvent): void {
  goal.zoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, goal.zoom * Math.exp(-event.deltaY * 0.0015)));
}

// Nouvel observateur : on y déplace le réticule et on recentre le globe.
watch(
  () => [props.latitudeDeg, props.longitudeDeg] as const,
  ([lat, lon]) => {
    placeReticle(lat, lon);
    goal.lat = Math.max(-89, Math.min(89, lat));
    goal.lon = lon;
  },
);

onMounted(() => {
  const host = container.value;
  if (!host) return;
  renderer = new WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  host.appendChild(renderer.domElement);
  scene = buildScene();
  camera = new PerspectiveCamera(CAMERA_FOV_DEG, 1, 0.01, 100);
  resize(host.clientWidth, host.clientHeight);
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
  pitch = null;
  earth = null;
  occluder = null;
  reticle = null;
  resizeObserver = null;
});
</script>

<template>
  <div
    ref="container"
    class="earth-picker"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @wheel.prevent="onWheel"
  />
</template>

<style scoped lang="scss">
.earth-picker {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  cursor: crosshair;
  touch-action: none;

  :deep(canvas) {
    display: block;
    width: 100%;
    height: 100%;
  }
}
</style>
