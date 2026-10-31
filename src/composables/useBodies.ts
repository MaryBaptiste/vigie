import { onUnmounted, ref, watch } from "vue";
import { getCometCatalogStatus, getVisibleBodies } from "../services/celestialBodyService";
import { toAppError } from "../services/invokeCommand";
import type { AppError } from "../types/appError";
import type { CelestialBody, CometCatalogStatus } from "../types/celestialBody";
import { useObserver } from "./useObserver";
import { useTimeControl } from "./useTimeControl";

const BODIES_PERIOD_MS = 5_000;
const COMET_STATUS_PERIOD_MS = 15_000;

/** Objets du système solaire au temps simulé, et état du catalogue des comètes. */
export function useBodies() {
  const time = useTimeControl();
  const { revision: observerRevision } = useObserver();

  const bodies = ref<CelestialBody[]>([]);
  const cometStatus = ref<CometCatalogStatus | null>(null);
  const error = ref<AppError | null>(null);
  const loading = ref(true);

  let inFlight = false;
  let generation = 0;

  async function refresh(): Promise<void> {
    if (inFlight) return;
    inFlight = true;
    const requestGeneration = generation;
    try {
      const result = await getVisibleBodies(new Date(time.simNowMs()).toISOString());
      if (requestGeneration !== generation) return;
      bodies.value = result;
      error.value = null;
    } catch (e) {
      error.value = toAppError(e);
    } finally {
      inFlight = false;
      loading.value = false;
    }
  }

  async function refreshCometStatus(): Promise<void> {
    try {
      cometStatus.value = await getCometCatalogStatus();
    } catch (e) {
      error.value = toAppError(e);
    }
  }

  watch([time.revision, observerRevision], () => {
    generation++;
    inFlight = false;
    void refresh();
  });

  void refresh();
  void refreshCometStatus();
  const bodiesTimer = setInterval(() => void refresh(), BODIES_PERIOD_MS);
  const cometTimer = setInterval(() => void refreshCometStatus(), COMET_STATUS_PERIOD_MS);
  onUnmounted(() => {
    clearInterval(bodiesTimer);
    clearInterval(cometTimer);
  });

  return { bodies, cometStatus, error, loading };
}
