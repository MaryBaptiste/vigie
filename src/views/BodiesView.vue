<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import BodyDetail from "../components/BodyDetail.vue";
import CometStatusPanel from "../components/CometStatusPanel.vue";
import TuiButton from "../components/TuiButton.vue";
import TuiPanel from "../components/TuiPanel.vue";
import TuiTable from "../components/TuiTable.vue";
import VisibilityLabel from "../components/VisibilityLabel.vue";
import { useBodies } from "../composables/useBodies";
import { useHotkeys } from "../composables/useHotkeys";
import { useObserver } from "../composables/useObserver";
import type { CelestialBody, Visibility } from "../types/celestialBody";
import type { TuiColumn } from "../types/table";
import { bodySymbol, bodyTypeLabel, shortBodyName } from "../utils/bodies";
import { fixed, localHourMinute } from "../utils/format";

const router = useRouter();
const { observer } = useObserver();
const { bodies, cometStatus, error, loading } = useBodies();
const selectedId = ref<string | null>(null);
const table = ref<{ moveSelection: (delta: number) => void } | null>(null);

const STATUS_RANK: Readonly<Record<Visibility, number>> = {
  VISIBLE: 0,
  "TROP FAIBLE": 1,
  "NOYÉ DANS LE JOUR": 2,
  "SOUS L'HORIZON": 3,
};

/** Visibles d'abord, puis par hauteur décroissante. */
function defaultOrder(a: CelestialBody, b: CelestialBody): number {
  return STATUS_RANK[a.status] - STATUS_RANK[b.status] || b.altitudeDeg - a.altitudeDeg;
}

const columns: readonly TuiColumn<CelestialBody>[] = [
  { key: "symbol", label: "", width: "2ch", align: "center" },
  { key: "name", label: "Nom", sortValue: (b) => b.name, text: (b) => shortBodyName(b.name).toUpperCase() },
  { key: "type", label: "Type", width: "9ch", sortValue: (b) => b.bodyType, text: (b) => bodyTypeLabel(b.bodyType) },
  { key: "status", label: "Statut", width: "17ch", sortValue: (b) => STATUS_RANK[b.status] },
  { key: "alt", label: "Alt", width: "6ch", align: "right", sortValue: (b) => b.altitudeDeg, text: (b) => fixed(b.altitudeDeg, 1, 5) },
  { key: "az", label: "Az", width: "6ch", align: "right", sortValue: (b) => b.azimuthDeg, text: (b) => fixed(b.azimuthDeg, 1, 5) },
  { key: "mag", label: "Mag", width: "6ch", align: "right", sortValue: (b) => b.magnitude, text: (b) => fixed(b.magnitude, 1, 5) },
  { key: "const", label: "Const.", width: "6ch", sortValue: (b) => b.constellation?.abbreviation ?? "", text: (b) => b.constellation?.abbreviation ?? "---" },
  { key: "rise", label: "Lever", width: "6ch", sortValue: (b) => b.rise ?? "~", text: (b) => localHourMinute(b.rise) },
  { key: "transit", label: "Passage", width: "8ch", sortValue: (b) => b.transit ?? "~", text: (b) => localHourMinute(b.transit) },
  { key: "set", label: "Coucher", width: "8ch", sortValue: (b) => b.set ?? "~", text: (b) => localHourMinute(b.set) },
];

const selected = computed(() => bodies.value.find((b) => b.id === selectedId.value) ?? null);
const brightComets = computed(() => bodies.value.filter((b) => b.bodyType === "COMET").length);
const visibleCount = computed(() => bodies.value.filter((b) => b.status === "VISIBLE").length);
const southern = computed(() => (observer.value?.latitudeDeg ?? 0) < 0);

// Première réponse : sélection du premier objet de la liste triée.
watch(bodies, (list) => {
  if (selectedId.value && list.some((b) => b.id === selectedId.value)) return;
  selectedId.value = [...list].sort(defaultOrder)[0]?.id ?? null;
});

function openInSky(): void {
  if (selected.value) void router.push({ name: "sky", query: { focus: selected.value.id } });
}

useHotkeys({
  arrowup: () => table.value?.moveSelection(-1),
  arrowdown: () => table.value?.moveSelection(1),
  enter: openInSky,
  v: openInSky,
});
</script>

<template>
  <main class="bodies-view">
    <TuiPanel title="Objets visibles" class="bodies-view__list">
      <template #status>
        <span v-if="error" class="bodies-view__error">ERREUR</span>
        <span v-else>{{ visibleCount }} VISIBLE(S) / {{ bodies.length }}</span>
      </template>
      <TuiTable
        ref="table"
        v-model:selected="selectedId"
        :columns="columns"
        :rows="bodies"
        :row-key="(b) => b.id"
        :default-order="defaultOrder"
      >
        <template #cell-symbol="{ row }">
          <span class="bodies-view__symbol">{{ bodySymbol(row.id, row.bodyType) }}</span>
        </template>
        <template #cell-status="{ row }">
          <VisibilityLabel :status="row.status" />
        </template>
        <template #empty>
          <div class="bodies-view__empty">{{ loading ? "CALCUL DES ÉPHÉMÉRIDES…" : (error?.message ?? "AUCUN OBJET") }}</div>
        </template>
      </TuiTable>
    </TuiPanel>

    <aside class="bodies-view__side">
      <TuiPanel :title="selected ? shortBodyName(selected.name) : 'Détail'" class="bodies-view__detail">
        <template #status>
          <TuiButton v-if="selected" hotkey="V" label="Voir dans le ciel" @click="openInSky" />
        </template>
        <BodyDetail v-if="selected" :body="selected" :southern="southern" />
      </TuiPanel>

      <TuiPanel title="Comètes">
        <CometStatusPanel :status="cometStatus" :bright-count="brightComets" />
      </TuiPanel>
    </aside>
  </main>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;

.bodies-view {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
  gap: $space-md;
  height: 100%;
  padding: $space-sm $space-md $space-md;
}

.bodies-view__list {
  min-width: 0;
}

.bodies-view__side {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
  min-height: 0;
}

.bodies-view__detail {
  flex: 1;
}

.bodies-view__detail :deep(.tui-panel__body) {
  overflow-y: auto;
  scrollbar-width: thin;
}

.bodies-view__detail :deep(.tui-panel__status) {
  pointer-events: auto;
}

.bodies-view__symbol {
  display: inline-block;
  width: 100%;
  text-align: center;
}

.bodies-view__error {
  color: $color-danger;
}

.bodies-view__empty {
  padding: $space-md;
  color: $color-dim;
}
</style>
