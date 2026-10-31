import { onMounted, onUnmounted } from "vue";

export type HotkeyMap = Readonly<Record<string, (event: KeyboardEvent) => void>>;

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

/**
 * Raccourcis clavier actifs tant que le composant est monté. Les clés sont
 * des `KeyboardEvent.key` en minuscules (« c », « f1 », « arrowup », « » pour
 * Espace). Ignorés pendant la saisie, sauf les touches de fonction.
 */
export function useHotkeys(map: HotkeyMap): void {
  function onKeydown(event: KeyboardEvent): void {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    const key = event.key.toLowerCase();
    const isFunctionKey = /^f\d+$/.test(key);
    if (!isFunctionKey && isTyping(event.target)) return;
    const handler = map[key];
    if (!handler) return;
    event.preventDefault();
    handler(event);
  }

  onMounted(() => window.addEventListener("keydown", onKeydown));
  onUnmounted(() => window.removeEventListener("keydown", onKeydown));
}
