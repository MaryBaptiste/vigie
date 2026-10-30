import { onMounted, onUnmounted, ref } from "vue";
import { getSolarState } from "../services/solarService";
import { toAppError } from "../services/invokeCommand";
import type { AppError } from "../types/appError";
import type { SolarState } from "../types/solar";

/** Le terminateur se déplace de 0,25° par minute : une mise à jour par minute suffit. */
const POLL_INTERVAL_MS = 60_000;

/** Point subsolaire et terminateur jour/nuit, rafraîchis chaque minute. */
export function useSolarState() {
  const solar = ref<SolarState | null>(null);
  const error = ref<AppError | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  async function poll(): Promise<void> {
    try {
      solar.value = await getSolarState();
      error.value = null;
    } catch (e) {
      error.value = toAppError(e);
    }
  }

  onMounted(() => {
    void poll();
    timer = setInterval(() => void poll(), POLL_INTERVAL_MS);
  });
  onUnmounted(() => clearInterval(timer));

  return { solar, error };
}
