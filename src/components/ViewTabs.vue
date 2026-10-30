<script setup lang="ts" generic="T extends string">
import { onMounted, onUnmounted } from "vue";

export interface ViewOption<Id extends string> {
  id: Id;
  label: string;
}

const props = defineProps<{
  options: readonly ViewOption<T>[];
  /** Touche qui fait défiler les onglets (ex. "v"). */
  cycleKey?: string;
}>();

const active = defineModel<T>({ required: true });

function cycle(): void {
  const index = props.options.findIndex((o) => o.id === active.value);
  const next = props.options[(index + 1) % props.options.length];
  if (next) active.value = next.id;
}

function onKeydown(event: KeyboardEvent): void {
  if (!props.cycleKey || event.key.toLowerCase() !== props.cycleKey.toLowerCase()) return;
  if (event.ctrlKey || event.altKey || event.metaKey) return;
  event.preventDefault();
  cycle();
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <span class="view-tabs" role="tablist">
    <button
      v-for="option in options"
      :key="option.id"
      type="button"
      role="tab"
      class="view-tabs__tab"
      :class="{ 'view-tabs__tab--active': option.id === active }"
      :aria-selected="option.id === active"
      @click="active = option.id"
    >
      {{ option.label }}
    </button>
    <span v-if="cycleKey" class="view-tabs__hint">[{{ cycleKey.toUpperCase() }}]</span>
  </span>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.view-tabs {
  display: inline-flex;
  gap: 1ch;
  align-items: baseline;
}

.view-tabs__tab {
  padding: 0 0.5ch;
  border: none;
  background: none;
  color: $color-dim;
  font: inherit;
  text-transform: inherit;
  letter-spacing: inherit;
  text-shadow: none;
  cursor: pointer;

  &:hover {
    color: $color-fg;
  }
}

.view-tabs__tab--active {
  @include inverse;

  &:hover {
    color: $color-bg;
  }
}

.view-tabs__hint {
  color: $color-faint;
  text-shadow: none;
}
</style>
