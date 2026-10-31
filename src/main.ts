import { createApp } from "vue";
// Police embarquée dans le bundle (aucun CDN : l'app doit tourner hors ligne).
import "@fontsource/vt323/400.css";
import "./styles/main.scss";
import App from "./App.vue";
import { router } from "./router";

createApp(App).use(router).mount("#app");
