import DefaultTheme from "vitepress/theme";
import "@fontsource/anonymous-pro/400.css";
import "@fontsource/anonymous-pro/400-italic.css";
import "@fontsource/anonymous-pro/700.css";
import "@fontsource/chivo-mono/400.css";
import "@fontsource/chivo-mono/500.css";
import "@fontsource/chivo-mono/600.css";
import "@fontsource/syne/500.css";
import "@fontsource/syne/600.css";
import "@fontsource/syne/700.css";
import Home from "./Home.vue";
import "./theme.css";

export default {
  extends: DefaultTheme,
  Layout: DefaultTheme.Layout,
  enhanceApp({ app }) {
    app.component("DootHome", Home);
  },
};
