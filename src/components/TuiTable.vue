<script setup lang="ts" generic="Row">
import { computed, nextTick, ref, watch } from "vue";
import type { TuiColumn, TuiSort } from "../types/table";

const props = defineProps<{
  columns: readonly TuiColumn<Row>[];
  rows: readonly Row[];
  rowKey: (row: Row) => string;
  /** Tri initial et tri appliqué quand aucune colonne n'est choisie. */
  defaultOrder?: (a: Row, b: Row) => number;
  rowClass?: (row: Row) => string;
}>();

const selectedKey = defineModel<string | null>("selected", { default: null });
const sort = ref<TuiSort | null>(null);
const body = ref<HTMLElement | null>(null);

const sortedRows = computed(() => {
  const rows = [...props.rows];
  const active = sort.value;
  const column = active ? props.columns.find((c) => c.key === active.key) : undefined;
  if (active && column?.sortValue) {
    const value = column.sortValue;
    const factor = active.direction === "asc" ? 1 : -1;
    return rows.sort((a, b) => {
      const va = value(a);
      const vb = value(b);
      return (typeof va === "number" && typeof vb === "number" ? va - vb : String(va).localeCompare(String(vb))) * factor;
    });
  }
  return props.defaultOrder ? rows.sort(props.defaultOrder) : rows;
});

/** Clic sur un en-tête : croissant, décroissant, puis retour au tri par défaut. */
function toggleSort(column: TuiColumn<Row>): void {
  if (!column.sortValue) return;
  const current = sort.value;
  if (current?.key !== column.key) sort.value = { key: column.key, direction: "asc" };
  else if (current.direction === "asc") sort.value = { key: column.key, direction: "desc" };
  else sort.value = null;
}

function sortMark(column: TuiColumn<Row>): string {
  if (sort.value?.key !== column.key) return column.sortValue ? " " : "";
  return sort.value.direction === "asc" ? "▲" : "▼";
}

/** Déplace la sélection (flèches du clavier), avec défilement. */
function moveSelection(delta: number): void {
  const rows = sortedRows.value;
  if (rows.length === 0) return;
  const index = rows.findIndex((r) => props.rowKey(r) === selectedKey.value);
  const next = rows[Math.max(0, Math.min(rows.length - 1, index < 0 ? 0 : index + delta))];
  if (next) selectedKey.value = props.rowKey(next);
}

watch(selectedKey, async () => {
  await nextTick();
  body.value?.querySelector(".tui-table__row--selected")?.scrollIntoView({ block: "nearest" });
});

defineExpose({ moveSelection });
</script>

<template>
  <div class="tui-table">
    <div class="tui-table__row tui-table__row--header">
      <button
        v-for="column in columns"
        :key="column.key"
        type="button"
        class="tui-table__cell tui-table__header"
        :class="[`tui-table__cell--${column.align ?? 'left'}`, { 'tui-table__header--sortable': column.sortValue }]"
        :style="{ width: column.width, flex: column.width ? 'none' : '1' }"
        @click="toggleSort(column)"
      >
        {{ column.label }}{{ sortMark(column) }}
      </button>
    </div>
    <div ref="body" class="tui-table__body">
      <div
        v-for="row in sortedRows"
        :key="rowKey(row)"
        class="tui-table__row"
        :class="[rowClass?.(row), { 'tui-table__row--selected': rowKey(row) === selectedKey }]"
        @click="selectedKey = rowKey(row)"
      >
        <span
          v-for="column in columns"
          :key="column.key"
          class="tui-table__cell"
          :class="`tui-table__cell--${column.align ?? 'left'}`"
          :style="{ width: column.width, flex: column.width ? 'none' : '1' }"
        >
          <slot :name="`cell-${column.key}`" :row="row">{{ column.text?.(row) ?? "" }}</slot>
        </span>
      </div>
      <slot v-if="sortedRows.length === 0" name="empty" />
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.tui-table {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.tui-table__body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: $color-dim transparent;
}

.tui-table__row {
  display: flex;
  gap: 1ch;
  padding: 0 0.5ch;
  cursor: pointer;

  &:hover:not(.tui-table__row--header) {
    background: $color-faint;
  }
}

.tui-table__row--header {
  border-bottom: 1px solid $color-dim;
  margin-bottom: $space-xs;
  cursor: default;
}

.tui-table__row--selected,
.tui-table__row--selected:hover {
  @include inverse;
}

.tui-table__cell {
  min-width: 0;
  @include single-line;
}

.tui-table__cell--right {
  text-align: right;
}

.tui-table__cell--center {
  text-align: center;
}

.tui-table__header {
  border: none;
  background: transparent;
  color: $color-dim;
  font: inherit;
  text-shadow: none;
  cursor: default;
  @include uppercase-label;
}

.tui-table__header--sortable {
  cursor: pointer;

  &:hover {
    color: $color-fg;
  }
}
</style>
