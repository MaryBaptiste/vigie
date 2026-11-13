<script setup lang="ts">
import { computed } from "vue";
import type { NightCurve } from "../types/celestialBody";
import { altitudeChart } from "../utils/ascii";
import { localHourMinute } from "../utils/format";

const props = defineProps<{ night: NightCurve }>();

const ROWS = 6;

const lines = computed(() => altitudeChart(props.night.altitudes, ROWS));
const width = computed(() => props.night.altitudes.length);

/** Meilleur moment : échantillon le plus haut de la nuit. */
const best = computed(() => {
  const altitudes = props.night.altitudes;
  const max = Math.max(...altitudes);
  if (!(max > 0)) return null;
  const index = altitudes.indexOf(max);
  const start = Date.parse(props.night.start);
  const end = Date.parse(props.night.end);
  const at = start + ((end - start) * index) / Math.max(1, altitudes.length - 1);
  return { index, label: localHourMinute(new Date(at).toISOString()), altitude: max };
});

const marker = computed(() => (best.value ? " ".repeat(best.value.index) + "▲" : ""));
const axis = computed(() => {
  const left = props.night.sunset ? `☾ ${localHourMinute(props.night.sunset)}` : localHourMinute(props.night.start);
  const right = props.night.sunrise ? `${localHourMinute(props.night.sunrise)} ☼` : localHourMinute(props.night.end);
  return left + right.padStart(Math.max(1, width.value - left.length), " ");
});
</script>

<template>
  <div class="altitude-chart">
    <pre class="altitude-chart__plot"><span
      v-for="(line, i) in lines"
      :key="i"
      class="altitude-chart__line"
    ><span class="altitude-chart__scale">{{ i === 0 ? "90°" : i === ROWS - 1 ? " 0°" : "   " }}│</span>{{ line }}
</span><span class="altitude-chart__scale">   └</span>{{ "─".repeat(width) }}
<span class="altitude-chart__scale">    </span><span class="altitude-chart__marker">{{ marker }}</span>
<span class="altitude-chart__scale">    </span>{{ axis }}</pre>
    <div v-if="best" class="altitude-chart__best">BEST TIME {{ best.label }} · {{ best.altitude.toFixed(0) }}°</div>
    <div v-else class="altitude-chart__best altitude-chart__best--none">NEVER ABOVE THE HORIZON TONIGHT</div>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.altitude-chart__plot {
  font: inherit;
  line-height: 1;
  overflow: hidden;
}

.altitude-chart__scale {
  color: $color-dim;
  text-shadow: none;
}

.altitude-chart__marker {
  color: $color-alert;
}

.altitude-chart__best {
  margin-top: $space-xs;
  color: $color-alert;
  @include uppercase-label;
}

.altitude-chart__best--none {
  color: $color-dim;
}
</style>
