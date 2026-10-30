import { onMounted, ref } from "vue";
import { getSpaceObjects } from "../services/spaceObjectService";
import { toAppError } from "../services/invokeCommand";
import type { AppError } from "../types/appError";
import type { SpaceObject } from "../types/spaceObject";

/** Objet sélectionné au démarrage : l'ISS. */
const DEFAULT_NORAD_ID = 25544;

/** Liste des objets suivis et état de sélection. */
export function useSpaceObjects() {
  const objects = ref<SpaceObject[]>([]);
  const selectedId = ref<number | null>(null);
  const error = ref<AppError | null>(null);

  async function load(): Promise<void> {
    try {
      objects.value = await getSpaceObjects();
      error.value = null;
      const fallback = objects.value.find((o) => o.noradId === DEFAULT_NORAD_ID) ?? objects.value[0];
      selectedId.value = fallback?.noradId ?? null;
    } catch (e) {
      error.value = toAppError(e);
    }
  }

  onMounted(load);

  return { objects, selectedId, error, reload: load };
}
