type Mermaid = (typeof import("mermaid"))["default"];

let mermaidPromise: Promise<Mermaid> | undefined;

export function loadMermaid(): Promise<Mermaid> {
  mermaidPromise ??= import("mermaid").then(({ default: mermaid }) => {
    mermaid.initialize({
      startOnLoad: false,
      suppressErrorRendering: true,
      securityLevel: "strict",
      theme: "base",
      darkMode: true,
      fontFamily: "IBM Plex Mono, monospace",
      flowchart: {
        htmlLabels: false,
        curve: "basis",
      },
      themeVariables: {
        background: "#071011",
        primaryColor: "#0b2022",
        primaryTextColor: "#d9f7f6",
        primaryBorderColor: "#57cbd0",
        secondaryColor: "#102a2c",
        secondaryTextColor: "#d9f7f6",
        secondaryBorderColor: "#2f858a",
        tertiaryColor: "#171b17",
        tertiaryTextColor: "#f2c879",
        tertiaryBorderColor: "#a77b36",
        lineColor: "#62d4d8",
        textColor: "#d9f7f6",
        edgeLabelBackground: "#071011",
        clusterBkg: "#091718",
        clusterBorder: "#2f858a",
        fontFamily: "IBM Plex Mono, monospace",
      },
    });
    return mermaid;
  });

  return mermaidPromise;
}
