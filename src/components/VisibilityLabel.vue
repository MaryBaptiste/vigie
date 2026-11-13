<script setup lang="ts">
import type { Visibility } from "../types/celestialBody";

defineProps<{ status: Visibility }>();

const TONES: Readonly<Record<Visibility, string>> = {
  VISIBLE: "visible",
  "BELOW HORIZON": "below",
  "LOST IN DAYLIGHT": "daylight",
  "TOO FAINT": "faint",
};
</script>

<template>
  <span class="visibility" :class="`visibility--${TONES[status]}`">{{ status }}</span>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.visibility--visible {
  color: $color-fg;
}

.visibility--below {
  color: $color-dim;
  text-shadow: none;
}

.visibility--daylight {
  color: $color-sun;
}

.visibility--faint {
  color: $color-faint;
  text-shadow: none;
}
</style>
