<script setup lang="ts">
import { ref } from "vue";
import { SPEEDS, useTimeControl } from "../composables/useTimeControl";
import { localDateTime, toDateTimeLocalValue } from "../utils/format";
import DataRow from "./DataRow.vue";
import TuiButton from "./TuiButton.vue";
import TuiInput from "./TuiInput.vue";

const time = useTimeControl();
const clock = time.useSimClock();
const dateInput = ref(toDateTimeLocalValue(time.simNowMs()));

function goToDate(): void {
  const ms = new Date(dateInput.value).getTime();
  if (Number.isFinite(ms)) time.setDate(ms);
}

function speedLabel(speed: number): string {
  return speed === 1 ? "▶ x1" : `x${speed}`;
}
</script>

<template>
  <div class="time-controls">
    <DataRow label="Temps" :value="localDateTime(clock)" />
    <DataRow label="Mode" :value="time.mode.value" :tone="time.live.value ? 'normal' : 'alert'" />
    <div class="time-controls__row">
      <TuiButton label="‖" :active="time.paused.value" @click="time.togglePause()" />
      <TuiButton
        v-for="speed in SPEEDS"
        :key="speed"
        :label="speedLabel(speed)"
        :active="!time.paused.value && time.speed.value === speed"
        @click="time.setSpeed(speed)"
      />
    </div>
    <div class="time-controls__row">
      <TuiButton label="-1h" @click="time.step(-1)" />
      <TuiButton label="+1h" @click="time.step(1)" />
      <TuiButton label="⟲ Maintenant" :active="time.live.value" @click="time.now()" />
    </div>
    <div class="time-controls__row">
      <TuiInput v-model="dateInput" label="Date" type="datetime-local" width="17ch" @submit="goToDate" />
      <TuiButton label="Aller" @click="goToDate" />
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.time-controls {
  display: flex;
  flex-direction: column;
}

.time-controls__row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0 0.5ch;
  margin-top: $space-xs;
}
</style>
