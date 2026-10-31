import { createRouter, createWebHashHistory } from "vue-router";
import BodiesView from "../views/BodiesView.vue";
import MainView from "../views/MainView.vue";
import SkyView from "../views/SkyView.vue";

declare module "vue-router" {
  interface RouteMeta {
    /** Écran du planétarium : la barre de navigation affiche le lieu observé et le mode de temps. */
    planetarium?: boolean;
  }
}

export const router = createRouter({
  // Historique par hash : fonctionne tel quel avec le protocole de Tauri.
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "orbit", component: MainView },
    { path: "/ciel", name: "sky", component: SkyView, meta: { planetarium: true } },
    { path: "/objets", name: "bodies", component: BodiesView, meta: { planetarium: true } },
  ],
});
