<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    type?: "text" | "number" | "datetime-local";
    width?: string;
    step?: string;
  }>(),
  { type: "text", width: "10ch", step: "any" },
);

const model = defineModel<string>({ required: true });
defineEmits<{ submit: [] }>();
</script>

<template>
  <label class="tui-input">
    <span class="tui-input__label">{{ label }}</span>
    <input
      :value="model"
      class="tui-input__field"
      :type="type"
      :step="type === 'number' ? step : undefined"
      :style="{ width }"
      spellcheck="false"
      @input="model = ($event.target as HTMLInputElement).value"
      @keydown.enter="$emit('submit')"
    />
  </label>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.tui-input {
  display: inline-flex;
  align-items: baseline;
  gap: 1ch;
}

.tui-input__label {
  color: $color-dim;
  @include uppercase-label;
}

.tui-input__field {
  padding: 0 0.5ch;
  border: none;
  border-bottom: 1px solid $color-dim;
  background: transparent;
  color: $color-fg;
  font: inherit;
  caret-color: $color-fg;
  color-scheme: dark;
  @include glow;

  &:focus {
    outline: none;
    border-bottom-color: $color-fg;
    background: $color-faint;
  }

  // Masque les flèches des champs numériques.
  &::-webkit-inner-spin-button,
  &::-webkit-outer-spin-button {
    appearance: none;
    margin: 0;
  }
}
</style>
