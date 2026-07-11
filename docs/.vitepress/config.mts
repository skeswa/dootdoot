import { defineConfig } from "vitepress";
import { sidebar } from "./navigation.mjs";

export default defineConfig({
  title: "dootdoot",
  description: "A deterministic, learnable sound-language for your terminal.",
  base: "/dootdoot/",
  cleanUrls: true,
  lastUpdated: true,
  vite: {
    publicDir: "../dootdoot-core/tests/fixtures/golden",
  },
  head: [
    ["meta", { name: "theme-color", content: "#0b0c0c" }],
    ["meta", { property: "og:title", content: "dootdoot — text, translated into droid" }],
    [
      "meta",
      {
        property: "og:description",
        content: "A deterministic, semantically-aware sound language.",
      },
    ],
  ],
  themeConfig: {
    nav: [
      { text: "Try it", link: "/#signal-console" },
      { text: "How it works", link: "/design" },
      { text: "Docs", link: "/README" },
    ],
    sidebar,
    outline: { level: [2, 3], label: "On this page" },
    search: { provider: "local" },
    editLink: {
      pattern: "https://github.com/skeswa/dootdoot/edit/main/docs/:path",
      text: "Edit this page",
    },
    socialLinks: [{ icon: "github", link: "https://github.com/skeswa/dootdoot" }],
    footer: {
      message: "Independent open-source droid acoustics. Not affiliated with Lucasfilm or Disney.",
      copyright: "MIT licensed.",
    },
  },
});
