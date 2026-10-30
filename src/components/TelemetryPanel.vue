<script setup lang="ts">
import { computed } from "vue";
import DataRow from "./DataRow.vue";
import TuiPanel from "./TuiPanel.vue";
import type { AppError } from "../types/appError";
import type { Telemetry } from "../types/telemetry";
import { fixed, integer, signed, utcDateTime, utcTime } from "../utils/format";

/** Au-delà, la précision SGP4 se dégrade nettement. */
const STALE_TLE_HOURS = 72;
const VALUE_WIDTH = 10;

const props = defineProps<{
  telemetry: Telemetry | null;
  error: AppError | null;
  objectName: string | null;
}>();

interface Row {
  label: string;
  value: string;
  unit?: string;
  alert?: boolean;
}

const isStale = computed(() => (props.telemetry?.tleAgeHours ?? 0) > STALE_TLE_HOURS);
const signalLost = computed(() => props.error !== null);

const sections = computed<Row[][]>(() => {
  const t = props.telemetry;
  if (!t) return [];
  const w = VALUE_WIDTH;
  return [
    [
      { label: "UTC", value: utcTime(t.timestamp).padStart(w) },
      { label: "Lat", value: signed(t.latitudeDeg, 4, w), unit: "°" },
      { label: "Lon", value: signed(t.longitudeDeg, 4, w), unit: "°" },
      { label: "Alt", value: fixed(t.altitudeKm, 2, w), unit: "km" },
      { label: "Vitesse", value: fixed(t.speedKmS, 3, w), unit: "km/s" },
    ],
    [
      { label: "Inclinaison", value: fixed(t.inclinationDeg, 3, w), unit: "°" },
      { label: "Période", value: fixed(t.periodMin, 2, w), unit: "min" },
      { label: "Apogée", value: fixed(t.apogeeKm, 1, w), unit: "km" },
      { label: "Périgée", value: fixed(t.perigeeKm, 1, w), unit: "km" },
      { label: "Excentricité", value: fixed(t.eccentricity, 7, w) },
      { label: "Révolution", value: integer(t.revolutionNumber, w), unit: "#" },
    ],
    [
      { label: "Époque TLE", value: utcDateTime(t.tleEpoch) },
      { label: "Âge TLE", value: fixed(t.tleAgeHours, 1, w), unit: "h", alert: isStale.value },
    ],
  ];
});

function rowTone(row: Row): "normal" | "alert" | "muted" {
  if (signalLost.value) return "muted";
  return row.alert ? "alert" : "normal";
}
</script>

<template>
  <TuiPanel :title="objectName ? `Télémétrie · ${objectName}` : 'Télémétrie'" class="telemetry-panel">
    <template #status>
      <span v-if="signalLost" class="status status--danger">SIGNAL LOST</span>
      <span v-else-if="isStale" class="status status--alert">DATA STALE</span>
      <span v-else-if="telemetry" class="status">LIVE</span>
      <span v-else class="status status--muted">ACQ</span>
    </template>

    <div v-if="telemetry" class="telemetry-panel__rows">
      <div v-for="(rows, i) in sections" :key="i" class="telemetry-panel__section">
        <DataRow
          v-for="row in rows"
          :key="row.label"
          :label="row.label"
          :value="row.value"
          :unit="row.unit"
          :tone="rowTone(row)"
        />
      </div>
    </div>

    <div v-else-if="error" class="telemetry-panel__message telemetry-panel__message--danger">
      <p class="telemetry-panel__headline">*** SIGNAL LOST ***</p>
      <p class="telemetry-panel__detail">{{ error.message }}</p>
      <p class="telemetry-panel__detail">Nouvelle tentative en cours<span class="cursor">_</span></p>
    </div>

    <div v-else class="telemetry-panel__message">
      <p>Acquisition du signal<span class="cursor">_</span></p>
    </div>

    <p v-if="telemetry && error" class="telemetry-panel__footer">
      Dernières valeurs connues · {{ error.message }}
    </p>
  </TuiPanel>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.telemetry-panel__section + .telemetry-panel__section {
  margin-top: $space-sm;
  padding-top: $space-sm;
  border-top: 1px dashed $color-faint;
}

.telemetry-panel__message {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: $space-sm;
  height: 100%;
  text-align: center;
}

.telemetry-panel__message--danger {
  color: $color-danger;
}

.telemetry-panel__headline {
  font-size: 1.5em;
  @include blink;
}

.telemetry-panel__detail {
  color: $color-dim;
  text-shadow: none;
  white-space: normal;
  overflow-wrap: anywhere;
}

.telemetry-panel__footer {
  margin-top: $space-sm;
  color: $color-danger;
  @include single-line;
  text-overflow: ellipsis;
}

.status--danger {
  color: $color-danger;
  @include blink;
}

.status--alert {
  color: $color-alert;
  @include blink;
}

.status--muted {
  color: $color-dim;
}

.cursor {
  @include blink(0.8s);
}
</style>
