<script setup lang="ts">
import { computed } from "vue";
import type { CometCatalogStatus } from "../types/celestialBody";
import { utcDateTime } from "../utils/format";
import DataRow from "./DataRow.vue";

const props = defineProps<{
  status: CometCatalogStatus | null;
  /** Comètes de magnitude ≤ 12 actuellement listées. */
  brightCount: number;
}>();

const SOURCES = {
  NETWORK: "NETWORK (MPC)",
  CACHE: "DISK CACHE",
  OFFLINE: "CACHE, OFFLINE",
  LOADING: "DOWNLOADING…",
} as const;

const updatedAt = computed(() => (props.status?.updatedAt ? `${utcDateTime(props.status.updatedAt)} UTC` : "NEVER"));
</script>

<template>
  <div class="comet-status">
    <div class="comet-status__flags">
      <span v-if="status?.source === 'OFFLINE'" class="comet-status__flag comet-status__flag--offline">OFFLINE</span>
      <span v-if="status?.stale" class="comet-status__flag comet-status__flag--stale">DATA STALE</span>
    </div>
    <DataRow label="MPC elements" :value="updatedAt" :tone="status?.stale ? 'alert' : 'normal'" />
    <DataRow label="Source" :value="status ? SOURCES[status.source] : '---'" />
    <DataRow label="Catalog" :value="String(status?.cometCount ?? 0)" unit="cmt" />
    <DataRow label="Mag ≤ 12" :value="String(brightCount)" unit="cmt" />
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.comet-status__flags {
  display: flex;
  gap: 1ch;
  min-height: 1em;
}

.comet-status__flag {
  padding: 0 1ch;
  @include uppercase-label;
}

.comet-status__flag--offline {
  background: $color-danger;
  color: $color-bg;
  text-shadow: none;
}

.comet-status__flag--stale {
  color: $color-alert;
  @include blink;
}
</style>
