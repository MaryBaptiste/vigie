import { readonly, ref } from "vue";
import { toAppError } from "../services/invokeCommand";
import { getObserver, setObserver as saveObserver } from "../services/observerService";
import type { AppError } from "../types/appError";
import type { Observer } from "../types/observer";

/** Observateur partagé par les deux écrans (singleton de module). */
const observer = ref<Observer | null>(null);
/** Incrémenté à chaque changement de position : les instantanés du ciel sont alors périmés. */
const revision = ref(0);
const error = ref<AppError | null>(null);
let loading: Promise<void> | null = null;

function load(): Promise<void> {
  loading ??= getObserver()
    .then((value) => {
      observer.value = value;
    })
    .catch((e: unknown) => {
      error.value = toAppError(e);
      loading = null;
    });
  return loading;
}

async function setObserver(value: Observer): Promise<void> {
  try {
    observer.value = await saveObserver(value);
    error.value = null;
    revision.value++;
  } catch (e) {
    error.value = toAppError(e);
  }
}

export function useObserver() {
  void load();
  return {
    observer: readonly(observer),
    revision: readonly(revision),
    error: readonly(error),
    setObserver,
  };
}
