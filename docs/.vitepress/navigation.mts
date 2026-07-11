import { readdirSync } from "node:fs";
import { resolve } from "node:path";

type SidebarItem = { text: string; link: string };

const docsRoot = resolve(import.meta.dirname, "..");
const title = (name: string) =>
  name
    .replace(/\.md$/, "")
    .replaceAll("-", " ")
    .replaceAll("_", " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());

const directory = (name: string): SidebarItem[] =>
  readdirSync(resolve(docsRoot, name), { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".md"))
    .sort((left, right) => left.name.localeCompare(right.name))
    .map((entry) => ({
      text: title(entry.name),
      link: `/${name}/${entry.name.replace(/\.md$/, "")}`,
    }));

export const sidebar = [
  {
    text: "Start here",
    items: [
      { text: "Documentation map", link: "/README" },
      { text: "Usage", link: "/usage" },
      { text: "Design", link: "/design" },
      { text: "Requirements", link: "/spec" },
      { text: "Build plan", link: "/plan" },
      { text: "Rust style", link: "/style" },
    ],
  },
  { text: "Reference", collapsed: false, items: directory("reference") },
  { text: "Research notes", collapsed: true, items: directory("research") },
  { text: "Voice validation", collapsed: true, items: directory("validation") },
];
