<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import DataRow from "../components/DataRow.vue";
import EarthPicker from "../components/EarthPicker.vue";
import ObserverPanel from "../components/ObserverPanel.vue";
import SkyDome from "../components/SkyDome.vue";
import SkyStatusBar from "../components/SkyStatusBar.vue";
import TimeControls from "../components/TimeControls.vue";
import TuiPanel from "../components/TuiPanel.vue";
import { useHotkeys } from "../composables/useHotkeys";
import { useObserver } from "../composables/useObserver";
import { useSky } from "../composables/useSky";
import { useTimeControl } from "../composables/useTimeControl";
import type { Observer } from "../types/observer";
import type { SkyPick } from "../types/skyPick";
import { cardinal, fixed, shortLatLon } from "../utils/format";

const route = useRoute();
const router = useRouter();
const time = useTimeControl();
const { observer, error: observerError, setObserver } = useObserver();
const { latest, error: skyError, source } = useSky();

const showConstellations = ref(true);
const showNames = ref(true);
const showGrid = ref(false);
const selected = ref<SkyPick | null>(null);
const dome = ref<InstanceType<typeof SkyDome> | null>(null);

/** Objet à centrer, transmis par la page Objets (`/#/ciel?focus=mars`). */
const focusId = computed(() => (typeof route.query.focus === "string" ? route.query.focus : null));

const title = computed(() => {
  const o = observer.value;
  return o ? `Ciel ─ ${o.label} ${shortLatLon(o.latitudeDeg, o.longitudeDeg)}` : "Ciel";
});

const status = computed(() => (skyError.value ? "ERREUR" : (latest.value?.skyState ?? "…")));

function onPick(latitudeDeg: number, longitudeDeg: number): void {
  void setObserver({ latitudeDeg, longitudeDeg, altitudeM: 0, label: "" });
}

function onObserverChange(value: Observer): void {
  void setObserver(value);
}

function clearFocus(): void {
  selected.value = null;
  dome.value?.resetView();
  if (focusId.value) void router.replace({ name: "sky" });
}

// Changement de lieu : l'objet épinglé n'a plus la même position.
watch(observer, () => (selected.value = null));

useHotkeys({
  c: () => (showConstellations.value = !showConstellations.value),
  n: () => (showNames.value = !showNames.value),
  g: () => (showGrid.value = !showGrid.value),
  r: clearFocus,
  escape: clearFocus,
  " ": () => time.togglePause(),
  arrowleft: () => time.step(-1),
  arrowright: () => time.step(1),
});
</script>

<template>
  <main class="sky-view">
    <TuiPanel class="sky-view__dome" :title="title">
      <template #status>
        <span :class="{ 'sky-view__error': skyError }">{{ status }}</span>
      </template>
      <SkyDome
        ref="dome"
        v-model:selected="selected"
        :source="source"
        :show-constellations="showConstellations"
        :show-names="showNames"
        :show-grid="showGrid"
        :focus-id="focusId"
        :southern="(observer?.latitudeDeg ?? 0) < 0"
      />
      <div v-if="skyError" class="sky-view__error-message">{{ skyError.message }}</div>
    </TuiPanel>

    <aside class="sky-view__side">
      <TuiPanel title="Observateur" class="sky-view__observer">
        <div class="sky-view__globe">
          <EarthPicker
            v-if="observer"
            :latitude-deg="observer.latitudeDeg"
            :longitude-deg="observer.longitudeDeg"
            @pick="onPick"
          />
        </div>
        <ObserverPanel :observer="observer" :error="observerError?.message ?? null" @change="onObserverChange" />
      </TuiPanel>

      <TuiPanel title="Temps">
        <TimeControls />
      </TuiPanel>

      <TuiPanel v-if="selected" title="Objet">
        <template #status>[ESC]</template>
        <DataRow label="Nom" :value="selected.name.toUpperCase()" />
        <DataRow label="Type" :value="selected.subtitle" tone="muted" />
        <DataRow label="Mag" :value="fixed(selected.magnitude, 2, 6)" />
        <DataRow label="Az" :value="`${fixed(selected.azimuthDeg, 2, 6)} ${cardinal(selected.azimuthDeg).padEnd(2)}`" unit="°" />
        <DataRow
          label="Alt"
          :value="fixed(selected.altitudeDeg, 2, 6)"
          unit="°"
          :tone="selected.altitudeDeg < 0 ? 'muted' : 'normal'"
        />
        <DataRow
          v-if="selected.illumination !== null"
          label="Illum."
          :value="fixed(selected.illumination * 100, 0, 3)"
          unit="%"
        />
      </TuiPanel>
    </aside>

    <SkyStatusBar
      class="sky-view__status"
      :sky="latest"
      :show-constellations="showConstellations"
      :show-names="showNames"
      :show-grid="showGrid"
    />
  </main>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.sky-view {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
  grid-template-rows: 1fr auto;
  gap: $space-sm $space-md;
  height: 100%;
  padding: $space-sm $space-md $space-sm;
}

.sky-view__dome {
  min-width: 0;
}

.sky-view__side {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
  min-height: 0;
  overflow: hidden;
}

.sky-view__observer {
  flex: 1;
  min-height: 12em;
}

.sky-view__observer :deep(.tui-panel__body) {
  display: flex;
  flex-direction: column;
}

.sky-view__globe {
  flex: 1;
  min-height: 6em;
}

.sky-view__status {
  grid-column: 1 / -1;
}

.sky-view__error {
  color: $color-danger;
}

.sky-view__error-message {
  position: absolute;
  bottom: $space-sm;
  left: 1.5ch;
  color: $color-danger;
}
</style>
