import { defineConfig } from "vitepress";
import { sidebar } from "./navigation.mjs";

export default defineConfig({
  title: "dootdoot",
  description: "A deterministic, learnable sound-language for your terminal.",
  base: "/dootdoot/",
  cleanUrls: true,
  lastUpdated: true,
  head: [
    ["meta", { name: "theme-color", content: "#05090a" }],
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
      { text: "Console", link: "/#signal-console" },
      { text: "Dialect", link: "/#dialect" },
      { text: "Manual", link: "/README" },
      { text: "GitHub ↗", link: "https://github.com/skeswa/dootdoot" },
    ],
    sidebar,
    outline: { level: [2, 3], label: "On this page" },
    search: { provider: "local" },
    editLink: {
      pattern: "https://github.com/skeswa/dootdoot/edit/main/docs/:path",
      text: "Edit this page",
    },
    footer: {
      message: "Independent open-source droid acoustics. Not affiliated with Lucasfilm or Disney.",
      copyright: "MIT licensed.",
    },
  },
});
