import { defineConfig } from "vitepress";
import { sidebar } from "./navigation.mjs";

export default defineConfig({
  title: "dootdoot",
  description: "A deterministic, learnable sound-language for your terminal.",
  base: "/dootdoot/",
  cleanUrls: true,
  lastUpdated: true,
  appearance: "force-dark",
  head: [
    ["meta", { name: "theme-color", content: "#05090a" }],
    [
      "link",
      {
        rel: "icon",
        href: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Cpath fill='%238fe8ee' d='M16 3 29 16 16 29 3 16Z'/%3E%3C/svg%3E",
      },
    ],
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
      { text: "GitHub ↗", link: "https://github.com/skeswa/dootdoot", noIcon: true },
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
