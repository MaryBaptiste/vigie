import type { Ref } from "vue";
import { getTelemetry } from "../services/telemetryService";
import { usePolling } from "./usePolling";

const POLL_INTERVAL_MS = 1000;

/** Télémétrie de l'objet sélectionné, rafraîchie chaque seconde. */
export function useTelemetry(selectedId: Ref<number | null>) {
  const { data: telemetry, error } = usePolling(selectedId, getTelemetry, POLL_INTERVAL_MS);
  return { telemetry, error };
}
