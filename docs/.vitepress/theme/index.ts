import DefaultTheme from "vitepress/theme";
import Home from "./Home.vue";
import "./style.css";

export default {
  extends: DefaultTheme,
  Layout: DefaultTheme.Layout,
  enhanceApp({ app }) {
    app.component("DootHome", Home);
  },
};
