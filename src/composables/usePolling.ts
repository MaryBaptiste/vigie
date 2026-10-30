import { onUnmounted, ref, watch, type Ref } from "vue";
import { toAppError } from "../services/invokeCommand";
import type { AppError } from "../types/appError";

/**
 * Interroge `fetcher(id)` à intervalle régulier pour l'identifiant courant.
 * Relancé à chaque changement d'identifiant, stoppé au démontage.
 */
export function usePolling<T>(
  selectedId: Ref<number | null>,
  fetcher: (id: number) => Promise<T>,
  intervalMs: number,
) {
  const data = ref<T | null>(null) as Ref<T | null>;
  const error = ref<AppError | null>(null);

  let stopCurrent: (() => void) | null = null;

  function stop(): void {
    stopCurrent?.();
    stopCurrent = null;
  }

  function start(id: number): void {
    stop();
    data.value = null;
    error.value = null;

    let active = true;
    // Le premier appel peut attendre CelesTrak : on n'empile pas les requêtes.
    let inFlight = false;

    async function poll(): Promise<void> {
      if (inFlight) return;
      inFlight = true;
      try {
        const result = await fetcher(id);
        if (!active) return;
        data.value = result;
        error.value = null;
      } catch (e) {
        if (!active) return;
        error.value = toAppError(e);
      } finally {
        inFlight = false;
      }
    }

    void poll();
    const timer = setInterval(() => void poll(), intervalMs);
    stopCurrent = () => {
      active = false;
      clearInterval(timer);
    };
  }

  watch(selectedId, (id) => (id === null ? stop() : start(id)), { immediate: true });

  onUnmounted(stop);

  return { data, error };
}
