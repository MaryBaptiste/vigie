<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import TuiPanel from "./TuiPanel.vue";
import type { AppError } from "../types/appError";
import type { SpaceObject } from "../types/spaceObject";

const props = defineProps<{
  objects: SpaceObject[];
  error: AppError | null;
}>();

const selectedId = defineModel<number | null>({ required: true });

/** Ligne surlignée (curseur clavier), distincte de la sélection validée. */
const cursor = ref(0);

const cursorObject = computed<SpaceObject | undefined>(() => props.objects[cursor.value]);

watch(
  [() => props.objects, selectedId],
  () => {
    const index = props.objects.findIndex((o) => o.noradId === selectedId.value);
    if (index >= 0) cursor.value = index;
  },
  { immediate: true },
);

function select(index: number): void {
  const object = props.objects[index];
  if (!object) return;
  cursor.value = index;
  selectedId.value = object.noradId;
}

function moveCursor(delta: number): void {
  const count = props.objects.length;
  if (count === 0) return;
  cursor.value = (cursor.value + delta + count) % count;
}

// Écoute globale : en mode kiosque il n'y a pas d'autre champ de saisie,
// la navigation doit marcher sans avoir à donner le focus à la liste.
function onKeydown(event: KeyboardEvent): void {
  switch (event.key) {
    case "ArrowUp":
      moveCursor(-1);
      break;
    case "ArrowDown":
      moveCursor(1);
      break;
    case "Enter":
      select(cursor.value);
      break;
    default:
      return;
  }
  event.preventDefault();
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <TuiPanel title="Objects">
    <template #status>{{ objects.length.toString().padStart(2, "0") }} TRACKED</template>

    <p v-if="error" class="object-list__error">SIGNAL LOST · {{ error.message }}</p>

    <template v-else>
      <ul class="object-list" role="listbox" aria-label="Tracked objects">
        <li
          v-for="(object, index) in objects"
          :key="object.noradId"
          class="object-list__item"
          :class="{
            'object-list__item--cursor': index === cursor,
            'object-list__item--selected': object.noradId === selectedId,
          }"
          role="option"
          :aria-selected="object.noradId === selectedId"
          @click="select(index)"
          @mouseenter="cursor = index"
        >
          <span class="object-list__marker">{{ object.noradId === selectedId ? ">" : " " }}</span>
          <span class="object-list__name">{{ object.shortName }}</span>
          <span class="object-list__norad">{{ object.noradId.toString().padStart(6, "0") }}</span>
        </li>
      </ul>
      <p class="object-list__description">{{ cursorObject?.description ?? "" }}</p>
    </template>
  </TuiPanel>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.object-list__item {
  display: flex;
  gap: 1ch;
  padding: 0 0.5ch;
  cursor: pointer;
  @include single-line;
}

.object-list__item--cursor {
  @include inverse;

  .object-list__norad {
    color: $color-bg;
  }
}

.object-list__marker {
  flex: none;
  width: 1ch;
}

.object-list__item--selected .object-list__marker {
  @include blink;
}

.object-list__name {
  flex: 1;
}

.object-list__norad {
  color: $color-dim;
}

.object-list__description {
  margin-top: $space-sm;
  padding-top: $space-xs;
  border-top: 1px dashed $color-faint;
  color: $color-dim;
  text-shadow: none;
  @include single-line;
  text-overflow: ellipsis;
}

.object-list__error {
  color: $color-danger;
  white-space: normal;
}
</style>
