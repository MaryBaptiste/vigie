<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    value: string;
    unit?: string;
    tone?: "normal" | "alert" | "muted";
  }>(),
  { unit: "", tone: "normal" },
);
</script>

<template>
  <div class="data-row" :class="`data-row--${tone}`">
    <span class="data-row__label">{{ label }}</span>
    <span class="data-row__leader" aria-hidden="true" />
    <span class="data-row__value">{{ value }}</span>
    <span class="data-row__unit">{{ unit }}</span>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.data-row {
  display: flex;
  align-items: baseline;
  gap: 1ch;
  @include single-line;
}

.data-row__label {
  flex: none;
  @include uppercase-label;
}

// Points de conduite « ........ » qui remplissent l'espace libre.
.data-row__leader {
  flex: 1;
  min-width: 2ch;
  overflow: hidden;
  color: $color-faint;
  text-shadow: none;

  &::before {
    content: "................................................................................";
  }
}

.data-row__value {
  flex: none;
  white-space: pre;
  text-align: right;
}

.data-row__unit {
  flex: none;
  width: 4ch;
  color: $color-dim;
}

.data-row--alert {
  color: $color-alert;
}

.data-row--muted {
  color: $color-dim;
  text-shadow: none;
}
</style>
