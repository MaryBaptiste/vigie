<script setup lang="ts">
import { computed } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import TuiButton from "./components/TuiButton.vue";
import { useHotkeys } from "./composables/useHotkeys";
import { useObserver } from "./composables/useObserver";
import { useTimeControl } from "./composables/useTimeControl";
import { shortLatLon } from "./utils/format";

const router = useRouter();
const route = useRoute();
const { observer } = useObserver();
const time = useTimeControl();

const location = computed(() => {
  const o = observer.value;
  return o ? `${o.label.toUpperCase()} ${shortLatLon(o.latitudeDeg, o.longitudeDeg)}` : "---";
});

useHotkeys({
  f1: () => void router.push({ name: "orbit" }),
  f2: () => void router.push({ name: "sky" }),
  f3: () => void router.push({ name: "bodies" }),
});
</script>

<template>
  <div class="app">
    <nav class="app__nav">
      <span class="app__brand">VIGIE</span>
      <TuiButton hotkey="F1" label="Orbit" :active="route.name === 'orbit'" @click="router.push({ name: 'orbit' })" />
      <TuiButton hotkey="F2" label="Sky" :active="route.name === 'sky'" @click="router.push({ name: 'sky' })" />
      <TuiButton hotkey="F3" label="Objects" :active="route.name === 'bodies'" @click="router.push({ name: 'bodies' })" />
      <span class="app__spacer" />
      <template v-if="route.meta.planetarium">
        <span class="app__location">{{ location }}</span>
        <span class="app__mode" :class="{ 'app__mode--simulated': !time.live.value }">{{ time.mode.value }}</span>
      </template>
    </nav>
    <RouterView class="app__view" />
  </div>
</template>

<style scoped lang="scss">
@use "./styles/variables" as *;
@use "./styles/mixins" as *;

.app {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.app__nav {
  display: flex;
  align-items: baseline;
  gap: 1ch;
  padding: $space-xs $space-md 0;
  @include single-line;
}

.app__brand {
  margin-right: 1ch;
  letter-spacing: 0.3em;
  color: $color-alert;
}

.app__spacer {
  flex: 1;
}

.app__location {
  color: $color-dim;
  text-shadow: none;
}

.app__mode--simulated {
  color: $color-alert;
  @include blink;
}

.app__view {
  flex: 1;
  min-height: 0;
}
</style>
