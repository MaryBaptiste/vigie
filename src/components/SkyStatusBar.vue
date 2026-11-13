<script setup lang="ts">
import { computed } from "vue";
import { useTimeControl } from "../composables/useTimeControl";
import type { Sky } from "../types/sky";
import { hoursToClock, localClock, utcClock } from "../utils/format";

const props = defineProps<{
  sky: Sky | null;
  showConstellations: boolean;
  showNames: boolean;
  showGrid: boolean;
}>();

/** Rapport jour sidéral / jour solaire moyen. */
const SIDEREAL_RATE = 1.002_737_909_35;

const time = useTimeControl();
const clock = time.useSimClock();

/** Temps sidéral local, extrapolé depuis le dernier instantané pour qu'il défile à la seconde. */
const sidereal = computed(() => {
  const sky = props.sky;
  if (!sky) return "--:--:--";
  const elapsedHours = (clock.value - Date.parse(sky.timestamp)) / 3_600_000;
  return hoursToClock(sky.localSiderealTimeHours + elapsedHours * SIDEREAL_RATE);
});

/** Temps solaire moyen à la longitude de l'observateur (l'heure légale du lieu n'est pas connue hors ligne). */
const meanSolar = computed(() => {
  const lon = props.sky?.observer.longitudeDeg;
  return lon === undefined ? "--:--:--" : utcClock(clock.value + (lon / 15) * 3_600_000);
});

const toggles = computed(() => [
  { key: "C", label: "CONST.", on: props.showConstellations },
  { key: "N", label: "NAMES", on: props.showNames },
  { key: "G", label: "GRID", on: props.showGrid },
]);
</script>

<template>
  <footer class="sky-status-bar">
    <span>SIDEREAL {{ sidereal }}</span>
    <span class="sky-status-bar__sep">│</span>
    <span>LOCAL {{ localClock(clock) }}</span>
    <span class="sky-status-bar__sep">│</span>
    <span>UTC {{ utcClock(clock) }}</span>
    <span class="sky-status-bar__sep">│</span>
    <span class="sky-status-bar__muted">MEAN SOL. {{ meanSolar }}</span>
    <span class="sky-status-bar__spacer" />
    <span v-for="t in toggles" :key="t.key" class="sky-status-bar__toggle" :class="{ 'sky-status-bar__toggle--on': t.on }">
      [{{ t.key }}] {{ t.label }}
    </span>
    <span class="sky-status-bar__muted">[R] VIEW</span>
  </footer>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.sky-status-bar {
  display: flex;
  gap: 1ch;
  padding: 0 1ch;
  @include single-line;
}

.sky-status-bar__sep,
.sky-status-bar__muted {
  color: $color-dim;
  text-shadow: none;
}

.sky-status-bar__spacer {
  flex: 1;
}

.sky-status-bar__toggle {
  color: $color-dim;
  text-shadow: none;
}

.sky-status-bar__toggle--on {
  color: $color-fg;
  @include glow;
}
</style>
