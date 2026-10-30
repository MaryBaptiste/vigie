import type { Ref } from "vue";
import { getTrajectory } from "../services/trajectoryService";
import { usePolling } from "./usePolling";

/**
 * La trace est exprimée en repère terrestre : elle glisse vers l'ouest au fil
 * du temps. Un rafraîchissement toutes les 30 s suffit à la garder juste.
 */
const POLL_INTERVAL_MS = 30_000;

/** Trajectoire (passée et future) de l'objet sélectionné. */
export function useTrajectory(selectedId: Ref<number | null>) {
  const { data: trajectory, error } = usePolling(selectedId, getTrajectory, POLL_INTERVAL_MS);
  return { trajectory, error };
}
