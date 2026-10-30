<script setup lang="ts">
defineProps<{
  /** Titre texte ; le slot `title` permet un contenu interactif (onglets). */
  title?: string;
}>();
</script>

<template>
  <section class="tui-panel">
    <header class="tui-panel__header">
      <h2 class="tui-panel__title">
        <slot name="title">{{ title }}</slot>
      </h2>
      <span v-if="$slots.status" class="tui-panel__status">
        <slot name="status" />
      </span>
    </header>
    <div class="tui-panel__body">
      <slot />
    </div>
  </section>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

// Cadre type box-drawing : bordure pleine, titre « incrusté » dans le trait
// supérieur, coins marqués par des équerres plus épaisses.
.tui-panel {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  margin-top: 0.6em;
  border: $border-width solid $color-dim;
  box-shadow:
    0 0 6px $color-glow,
    inset 0 0 6px $color-glow;

  &::before,
  &::after {
    content: "";
    position: absolute;
    width: 0.8em;
    height: 0.8em;
    pointer-events: none;
    border-color: $color-fg;
    border-style: solid;
  }

  &::before {
    top: -$border-width;
    left: -$border-width;
    border-width: $border-width 0 0 $border-width;
  }

  &::after {
    right: -$border-width;
    bottom: -$border-width;
    border-width: 0 $border-width $border-width 0;
  }
}

.tui-panel__header {
  position: absolute;
  top: 0;
  left: 1ch;
  right: 1ch;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transform: translateY(-55%);
  pointer-events: none;
  z-index: 1;
}

.tui-panel__title,
.tui-panel__status {
  padding: 0 1ch;
  background: $color-bg;
  font-size: 1em;
  font-weight: normal;
  line-height: 1;
  @include uppercase-label;
  @include single-line;
}

// Le titre peut contenir des contrôles cliquables (onglets).
.tui-panel__title {
  pointer-events: auto;
}

.tui-panel__body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: $space-md 1.5ch $space-sm;
}
</style>
