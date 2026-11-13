<script setup lang="ts">
import { computed } from "vue";
import type { CelestialBody } from "../types/celestialBody";
import { bodyTypeLabel } from "../utils/bodies";
import { cardinal, fixed, localHourMinute, rightAscension, sexagesimal } from "../utils/format";
import AltitudeChart from "./AltitudeChart.vue";
import DataRow from "./DataRow.vue";
import MoonPhaseAscii from "./MoonPhaseAscii.vue";

const props = defineProps<{
  body: CelestialBody;
  southern: boolean;
}>();

const distance = computed(() =>
  props.body.distanceKm !== null
    ? { value: Math.round(props.body.distanceKm).toLocaleString("en-US"), unit: "km" }
    : { value: fixed(props.body.distanceAu, 4, 8), unit: "AU" },
);

const transit = computed(() => {
  const b = props.body;
  if (!b.transit) return "--:--";
  const altitude = b.transitAltitudeDeg === null ? "" : ` ${fixed(b.transitAltitudeDeg, 0, 3)}°`;
  return `${localHourMinute(b.transit)}${altitude}`;
});
</script>

<template>
  <div class="body-detail">
    <DataRow label="Type" :value="bodyTypeLabel(body.bodyType)" />
    <DataRow label="Status" :value="body.status" :tone="body.status === 'VISIBLE' ? 'normal' : 'muted'" />
    <DataRow label="Altitude" :value="fixed(body.altitudeDeg, 2, 7)" unit="°" />
    <DataRow label="Azimuth" :value="`${fixed(body.azimuthDeg, 2, 7)} ${cardinal(body.azimuthDeg).padEnd(2)}`" unit="°" />
    <DataRow label="Right asc." :value="rightAscension(body.rightAscensionDeg)" />
    <DataRow label="Declination" :value="sexagesimal(body.declinationDeg)" />
    <DataRow label="Magnitude" :value="fixed(body.magnitude, 2, 6)" />
    <DataRow label="Distance" :value="distance.value" :unit="distance.unit" />
    <DataRow
      v-if="body.heliocentricDistanceAu !== null"
      label="Sun dist."
      :value="fixed(body.heliocentricDistanceAu, 4, 8)"
      unit="AU"
    />
    <DataRow label="Elongation" :value="fixed(body.elongationDeg, 1, 6)" unit="°" />
    <DataRow label="Constellation" :value="body.constellation?.latinName.toUpperCase() ?? '---'" />
    <DataRow label="Rise" :value="localHourMinute(body.rise)" />
    <DataRow label="Transit" :value="transit" />
    <DataRow label="Set" :value="localHourMinute(body.set)" />
    <DataRow
      label="Max alt. night"
      :value="body.maxNightAltitudeDeg === null ? '---' : fixed(body.maxNightAltitudeDeg, 1, 6)"
      unit="°"
    />
    <DataRow v-if="body.illumination !== null" label="Illumination" :value="fixed(body.illumination * 100, 1, 6)" unit="%" />
    <DataRow v-if="body.phaseName" label="Phase" :value="body.phaseName" />

    <MoonPhaseAscii
      v-if="body.bodyType === 'MOON' && body.phaseAngleDeg !== null"
      class="body-detail__moon"
      :phase-angle-deg="body.phaseAngleDeg"
      :waxing="body.waxing ?? true"
      :southern="southern"
    />

    <h3 class="body-detail__heading">Altitude tonight</h3>
    <AltitudeChart :night="body.night" />
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.body-detail {
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.body-detail__moon {
  margin: $space-sm 0;
}

.body-detail__heading {
  margin-top: $space-sm;
  font-size: 1em;
  font-weight: normal;
  color: $color-dim;
  @include uppercase-label;
}
</style>
