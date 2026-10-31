<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    /** Touche affichée en préfixe, ex. `F1`. */
    hotkey?: string;
    active?: boolean;
    disabled?: boolean;
  }>(),
  { hotkey: "", active: false, disabled: false },
);

defineEmits<{ click: [event: MouseEvent] }>();
</script>

<template>
  <button
    type="button"
    class="tui-button"
    :class="{ 'tui-button--active': active }"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <span v-if="hotkey" class="tui-button__key">[{{ hotkey }}]</span>
    <span class="tui-button__label">{{ label }}</span>
  </button>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.tui-button {
  display: inline-flex;
  gap: 0.5ch;
  padding: 0 0.5ch;
  border: none;
  background: transparent;
  color: $color-fg;
  font: inherit;
  cursor: pointer;
  @include glow;
  @include uppercase-label;
  @include single-line;

  &::before {
    content: "[";
    color: $color-dim;
  }

  &::after {
    content: "]";
    color: $color-dim;
  }

  &:hover:not(:disabled),
  &:focus-visible {
    outline: none;
    color: $color-alert;
  }

  &:disabled {
    color: $color-faint;
    cursor: default;
    text-shadow: none;
  }
}

// Le crochet extérieur suffit quand une touche est affichée.
.tui-button:has(.tui-button__key) {
  &::before,
  &::after {
    content: none;
  }
}

.tui-button__key {
  color: $color-dim;
}

.tui-button--active .tui-button__label {
  @include inverse;
  padding: 0 0.5ch;
}
</style>
