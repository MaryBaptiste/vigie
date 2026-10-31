<script setup lang="ts">
import { ref, watch } from "vue";
import type { Observer } from "../types/observer";
import { PRESET_CITIES } from "../utils/cities";
import { fixed, hemisphere } from "../utils/format";
import DataRow from "./DataRow.vue";
import TuiButton from "./TuiButton.vue";
import TuiInput from "./TuiInput.vue";

const props = defineProps<{
  observer: Observer | null;
  error: string | null;
}>();

const emit = defineEmits<{ change: [observer: Observer] }>();

const editing = ref(false);
const form = ref({ lat: "", lon: "", alt: "", label: "" });
const formError = ref<string | null>(null);

function openForm(): void {
  const o = props.observer;
  form.value = {
    lat: o ? o.latitudeDeg.toFixed(4) : "",
    lon: o ? o.longitudeDeg.toFixed(4) : "",
    alt: o ? o.altitudeM.toFixed(0) : "0",
    label: o?.label ?? "",
  };
  formError.value = null;
  editing.value = true;
}

/** Accepte la virgule décimale française. */
function parse(text: string): number {
  return Number.parseFloat(text.trim().replace(",", "."));
}

function submit(): void {
  const latitudeDeg = parse(form.value.lat);
  const longitudeDeg = parse(form.value.lon);
  const altitudeM = form.value.alt.trim() === "" ? 0 : parse(form.value.alt);
  if (!(Math.abs(latitudeDeg) <= 90)) {
    formError.value = "LATITUDE ENTRE -90 ET 90";
    return;
  }
  if (!(Math.abs(longitudeDeg) <= 180)) {
    formError.value = "LONGITUDE ENTRE -180 ET 180";
    return;
  }
  if (!Number.isFinite(altitudeM)) {
    formError.value = "ALTITUDE INVALIDE";
    return;
  }
  emit("change", { latitudeDeg, longitudeDeg, altitudeM, label: form.value.label.trim() });
  editing.value = false;
}

watch(
  () => props.observer,
  () => (editing.value = false),
);
</script>

<template>
  <div class="observer-panel">
    <template v-if="observer">
      <DataRow label="Lieu" :value="observer.label.toUpperCase()" />
      <DataRow label="Lat" :value="hemisphere(observer.latitudeDeg, 'N', 'S', 2, 2)" />
      <DataRow label="Lon" :value="hemisphere(observer.longitudeDeg, 'E', 'O', 2, 3)" />
      <DataRow label="Alt" :value="fixed(observer.altitudeM, 0, 5)" unit="m" />
    </template>
    <DataRow v-else label="Position" value="---" tone="muted" />

    <form v-if="editing" class="observer-panel__form" @submit.prevent="submit">
      <TuiInput v-model="form.lat" label="Lat" type="number" width="9ch" @submit="submit" />
      <TuiInput v-model="form.lon" label="Lon" type="number" width="9ch" @submit="submit" />
      <TuiInput v-model="form.alt" label="Alt m" type="number" width="6ch" @submit="submit" />
      <TuiInput v-model="form.label" label="Nom" width="14ch" @submit="submit" />
      <div class="observer-panel__actions">
        <TuiButton label="OK" @click="submit" />
        <TuiButton label="Annuler" @click="editing = false" />
      </div>
      <div v-if="formError" class="observer-panel__error">{{ formError }}</div>
    </form>

    <div v-else class="observer-panel__presets">
      <TuiButton
        v-for="city in PRESET_CITIES"
        :key="city.label"
        :label="city.label"
        :active="observer?.label === city.label"
        @click="emit('change', city)"
      />
      <TuiButton label="Saisie" @click="openForm" />
    </div>
    <div v-if="error" class="observer-panel__error">{{ error }}</div>
  </div>
</template>

<style scoped lang="scss">
@use "../styles/variables" as *;
@use "../styles/mixins" as *;

.observer-panel {
  display: flex;
  flex-direction: column;
}

.observer-panel__presets,
.observer-panel__form {
  display: flex;
  flex-wrap: wrap;
  gap: 0 1ch;
  margin-top: $space-xs;
}

.observer-panel__actions {
  display: flex;
  gap: 1ch;
}

.observer-panel__error {
  color: $color-danger;
  @include uppercase-label;
}
</style>
