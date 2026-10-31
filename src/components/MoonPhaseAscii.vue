<script setup lang="ts">
import { computed } from "vue";
import { moonPhaseAscii } from "../utils/ascii";

const props = defineProps<{
  phaseAngleDeg: number;
  waxing: boolean;
  /** Hémisphère sud : la Lune apparaît retournée. */
  southern: boolean;
}>();

const lines = computed(() => moonPhaseAscii(props.phaseAngleDeg, props.waxing, props.southern));
</script>

<template>
  <pre class="moon-phase" aria-hidden="true">{{ lines.join("\n") }}</pre>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.moon-phase {
  font: inherit;
  font-size: 0.55em;
  line-height: 1;
  text-align: center;
  color: $color-fg;
}
</style>
