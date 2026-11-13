<script setup lang="ts">
import { computed, ref } from "vue";
import EarthGlobe from "../components/EarthGlobe.vue";
import ObjectList from "../components/ObjectList.vue";
import TelemetryPanel from "../components/TelemetryPanel.vue";
import TuiPanel from "../components/TuiPanel.vue";
import ViewTabs, { type ViewOption } from "../components/ViewTabs.vue";
import WorldMap from "../components/WorldMap.vue";
import { useSolarState } from "../composables/useSolarState";
import { useSpaceObjects } from "../composables/useSpaceObjects";
import { useTelemetry } from "../composables/useTelemetry";
import { useTrajectory } from "../composables/useTrajectory";
import { latLon } from "../utils/format";

type VisualizationView = "globe" | "map";

const VIEWS: readonly ViewOption<VisualizationView>[] = [
  { id: "globe", label: "Globe" },
  { id: "map", label: "World map" },
];

const view = ref<VisualizationView>("globe");

const { objects, selectedId, error: objectsError } = useSpaceObjects();
const { telemetry, error: telemetryError } = useTelemetry(selectedId);
const { trajectory } = useTrajectory(selectedId);
const { solar } = useSolarState();

const selectedName = computed(
  () => objects.value.find((o) => o.noradId === selectedId.value)?.shortName ?? null,
);

const visualizationStatus = computed(() => {
  if (view.value === "globe") return "WGS84 · Tilt 23.4°";
  return solar.value
    ? `Sun ${latLon(solar.value.subsolarLatitudeDeg, solar.value.subsolarLongitudeDeg)}`
    : "Sun ---";
});
</script>

<template>
  <main class="main-view">
    <TuiPanel class="main-view__visualization">
      <template #title>
        <ViewTabs v-model="view" :options="VIEWS" cycle-key="v" />
      </template>
      <template #status>{{ visualizationStatus }}</template>

      <!-- v-if : le globe est démonté (et sa scène WebGL libérée) quand il n'est pas affiché. -->
      <EarthGlobe v-if="view === 'globe'" :telemetry="telemetry" :trajectory="trajectory" />
      <WorldMap
        v-else
        :telemetry="telemetry"
        :trajectory="trajectory"
        :solar="solar"
        :object-name="selectedName"
      />
    </TuiPanel>

    <aside class="main-view__side">
      <ObjectList v-model="selectedId" :objects="objects" :error="objectsError" />
      <TelemetryPanel
        class="main-view__telemetry"
        :telemetry="telemetry"
        :error="telemetryError"
        :object-name="selectedName"
      />
    </aside>
  </main>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.main-view {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: $space-md;
  height: 100%;
  padding: $space-sm $space-md $space-md;
}

.main-view__visualization,
.main-view__side {
  min-width: 0;
  min-height: 0;
}

.main-view__side {
  display: flex;
  flex-direction: column;
  gap: $space-md;
}

.main-view__telemetry {
  flex: 1;
}
</style>
