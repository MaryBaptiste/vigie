import { computed, onUnmounted, readonly, ref } from "vue";

/** Accélérations proposées (1 = temps réel). */
export const SPEEDS = [1, 10, 60, 600, 3600] as const;
export type Speed = (typeof SPEEDS)[number];

const HOUR_MS = 3_600_000;

/**
 * Temps simulé partagé par toute l'application (singleton de module) :
 * un couple d'ancrage (instant réel, instant simulé) et une vitesse.
 */
const speed = ref<Speed>(1);
const paused = ref(false);
/** Suit l'horloge système (temps réel, non décalé). */
const live = ref(true);
/** Incrémenté à chaque saut (pas d'une heure, date choisie, « maintenant »). */
const revision = ref(0);

let anchorRealMs = performance.now();
let anchorSimMs = Date.now();

/** Instant simulé, en ms depuis l'epoch. Lu à chaque image : non réactif. */
function simNowMs(): number {
  if (paused.value) return anchorSimMs;
  return anchorSimMs + (performance.now() - anchorRealMs) * speed.value;
}

function reanchor(): void {
  anchorSimMs = simNowMs();
  anchorRealMs = performance.now();
}

function jumpTo(simMs: number): void {
  anchorSimMs = simMs;
  anchorRealMs = performance.now();
  live.value = false;
  revision.value++;
}

function setSpeed(value: Speed): void {
  reanchor();
  speed.value = value;
  paused.value = false;
  live.value = live.value && value === 1;
}

function togglePause(): void {
  reanchor();
  paused.value = !paused.value;
  live.value = false;
}

function step(hours: number): void {
  jumpTo(simNowMs() + hours * HOUR_MS);
}

function setDate(simMs: number): void {
  jumpTo(simMs);
}

function now(): void {
  anchorSimMs = Date.now();
  anchorRealMs = performance.now();
  speed.value = 1;
  paused.value = false;
  live.value = true;
  revision.value++;
}

/** Horloge simulée réactive, rafraîchie quatre fois par seconde pour l'affichage. */
function useSimClock(intervalMs = 250) {
  const nowMs = ref(simNowMs());
  const timer = setInterval(() => (nowMs.value = simNowMs()), intervalMs);
  onUnmounted(() => clearInterval(timer));
  return readonly(nowMs);
}

export function useTimeControl() {
  return {
    speed: readonly(speed),
    paused: readonly(paused),
    live: readonly(live),
    revision: readonly(revision),
    mode: computed(() => (paused.value ? "PAUSE" : live.value ? "REAL TIME" : `x${speed.value}`)),
    simNowMs,
    setSpeed,
    togglePause,
    step,
    setDate,
    now,
    useSimClock,
  };
}
