import { createHash } from "node:crypto";

export function renderMermaidDiagrams(markdown) {
  const renderFence = markdown.renderer.rules.fence;

  markdown.renderer.rules.fence = (tokens, index, options, environment, renderer) => {
    const token = tokens[index];
    if (token.info.trim().split(/\s+/u)[0] !== "mermaid") {
      return renderFence(tokens, index, options, environment, renderer);
    }

    const page = typeof environment?.path === "string" ? environment.path : "page";
    const digest = createHash("sha256")
      .update(`${page}\0${index}\0${token.content}`)
      .digest("hex")
      .slice(0, 12);
    const source = encodeURIComponent(token.content.trim());

    return `<MermaidDiagram diagram-id="mermaid-${digest}" source="${source}"></MermaidDiagram>\n`;
  };
}
