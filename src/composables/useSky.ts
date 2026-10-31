import { onUnmounted, ref, shallowRef, watch } from "vue";
import { toAppError } from "../services/invokeCommand";
import { getSky } from "../services/skyService";
import type { AppError } from "../types/appError";
import type { Sky } from "../types/sky";
import {
  interpolate,
  pairFrames,
  prepareFrame,
  type FramePair,
  type PreparedFrame,
  type RenderBuffers,
} from "../utils/skyInterpolation";
import { useObserver } from "./useObserver";
import { useTimeControl } from "./useTimeControl";

const SCHEDULER_PERIOD_MS = 100;
/** Délai de réponse supposé du backend, pour viser un peu en avance. */
const LATENCY_MS = 150;
/** Extrapolation maximale au-delà du dernier instantané (fraction d'intervalle). */
const MAX_EXTRAPOLATION = 1.5;

/** Intervalle réel entre deux instantanés selon l'accélération. */
function realIntervalMs(speed: number): number {
  if (speed >= 600) return 400;
  if (speed >= 60) return 500;
  if (speed >= 10) return 1000;
  return 5000;
}

/** Source du ciel pour la voûte : directions interpolées à un instant simulé. */
export interface SkySource {
  simNowMs: () => number;
  /** Remplit `out` pour l'instant `nowMs` ; renvoie l'instantané de référence. */
  frameAt: (nowMs: number, out: RenderBuffers) => PreparedFrame | null;
}

/**
 * Demande des instantanés en avance sur le temps simulé (un intervalle
 * d'avance), et interpole entre eux à chaque image.
 */
export function useSky() {
  const time = useTimeControl();
  const { revision: observerRevision } = useObserver();

  /** Dernier instantané de référence affiché (métadonnées : état du ciel, sidéral…). */
  const latest = shallowRef<Sky | null>(null);
  const error = ref<AppError | null>(null);

  let frames: PreparedFrame[] = [];
  let pair: FramePair | null = null;
  let inFlight = false;
  let generation = 0;

  async function request(timeMs: number): Promise<void> {
    inFlight = true;
    const requestGeneration = generation;
    try {
      const sky = await getSky(new Date(timeMs).toISOString());
      if (requestGeneration !== generation) return;
      const frame = prepareFrame(sky);
      frames = [...frames.filter((f) => f.timeMs !== frame.timeMs), frame].sort((a, b) => a.timeMs - b.timeMs);
      error.value = null;
    } catch (e) {
      if (requestGeneration === generation) error.value = toAppError(e);
    } finally {
      if (requestGeneration === generation) inFlight = false;
    }
  }

  function schedule(): void {
    if (inFlight) return;
    const now = time.simNowMs();
    const speed = time.speed.value;
    const step = speed * realIntervalMs(speed);

    // On garde l'instantané qui précède le précédent, utile pour extrapoler.
    const before = frames.filter((f) => f.timeMs <= now).length;
    if (before > 2) frames = frames.slice(before - 2);

    const last = frames.at(-1);
    if (!last) {
      void request(now);
    } else if (time.paused.value) {
      if (!frames.some((f) => Math.abs(f.timeMs - now) < 1000)) void request(now);
    } else if (last.timeMs < now - step) {
      // Trop en retard (backend lent, réveil de l'onglet) : on repart d'un point devant nous.
      void request(now + LATENCY_MS * speed);
    } else if (last.timeMs < now + step) {
      void request(last.timeMs + step);
    }
  }

  function reset(): void {
    generation++;
    frames = [];
    pair = null;
    inFlight = false;
    schedule();
  }

  function frameAt(nowMs: number, out: RenderBuffers): PreparedFrame | null {
    if (frames.length === 0) return null;
    let nextIndex = frames.findIndex((f) => f.timeMs > nowMs);
    if (nextIndex === 0) nextIndex = frames.length > 1 ? 1 : 0;
    // Pas d'instantané après « maintenant » : on extrapole depuis les deux derniers.
    const toIndex = nextIndex === -1 ? frames.length - 1 : nextIndex;
    const to = frames[toIndex] ?? null;
    if (!to) return null;
    const from = toIndex > 0 ? (frames[toIndex - 1] ?? null) : null;

    if (!pair || pair.to !== to || pair.from !== from) pair = pairFrames(from, to);
    const span = from ? to.timeMs - from.timeMs : 0;
    const fraction = from && span > 0 ? Math.min(MAX_EXTRAPOLATION, Math.max(0, (nowMs - from.timeMs) / span)) : 1;
    interpolate(pair, fraction, out);
    if (latest.value !== to.sky) latest.value = to.sky;
    return to;
  }

  watch([time.revision, time.speed, observerRevision], reset);
  const timer = setInterval(schedule, SCHEDULER_PERIOD_MS);
  schedule();
  onUnmounted(() => {
    clearInterval(timer);
    generation++;
  });

  const source: SkySource = { simNowMs: time.simNowMs, frameAt };
  return { latest, error, source };
}
