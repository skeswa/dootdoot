import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const docsRoot = resolve(import.meta.dirname, "..");

function heading(path) {
  const title = readFileSync(path, "utf8").match(/^#\s+(.+)$/m)?.[1];
  if (!title) throw new Error(`documentation page has no level-one heading: ${path}`);
  return title.replaceAll("`", "");
}

function directory(name) {
  return readdirSync(resolve(docsRoot, name), { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".md"))
    .map((entry) => ({
      text: heading(resolve(docsRoot, name, entry.name)),
      link: `/${name}/${entry.name.replace(/\.md$/, "")}`,
    }))
    .sort((left, right) => left.text.localeCompare(right.text, "en", { numeric: true }));
}

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
