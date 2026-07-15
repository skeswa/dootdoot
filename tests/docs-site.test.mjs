import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import test from "node:test";
import { parse } from "yaml";
import { sidebar } from "../docs/.vitepress/navigation.mjs";
import { samples } from "../docs/.vitepress/theme/samples.mjs";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const mermaidDiagrams = () => {
  const markdown = `${read("docs/design.md")}\n${read("docs/plan.md")}`;
  return [...markdown.matchAll(/```mermaid\n([\s\S]*?)```/g)].map(([, diagram]) => diagram);
};

test("the production site exposes its reader and playground", () => {
  const home = read("docs/.vitepress/dist/index.html");

  assert.match(home, /id="signal-console"/);
  assert.match(home, /MAKE IT DOOT/);
  assert.match(home, /placeholder="Type something\. Anything\."/);
  assert.match(home, /THE SAME RUST VOICE RUNS HERE AND IN THE CLI/);
  assert.equal(existsSync(new URL("../docs/.vitepress/dist/design.html", import.meta.url)), true);
  const assets = readdirSync(new URL("../docs/.vitepress/dist/assets/", import.meta.url));
  assert.ok(assets.some((file) => file.endsWith(".wasm")));
});

test("the production reader renders Mermaid diagrams", () => {
  const design = read("docs/.vitepress/dist/design.html");
  const plan = read("docs/.vitepress/dist/plan.html");

  assert.equal(design.match(/<figure class="mermaid-diagram\b/g)?.length, 1);
  assert.equal(plan.match(/<figure class="mermaid-diagram\b/g)?.length, 7);
  assert.equal(plan.match(/mermaid-diagram--wide/g)?.length, 7);
  assert.doesNotMatch(`${design}\n${plan}`, /language-mermaid/);
});

test("every Mermaid diagram describes itself to assistive technology", () => {
  const diagrams = mermaidDiagrams();

  assert.equal(diagrams.length, 8);
  for (const diagram of diagrams) {
    assert.match(diagram, /^\s*accTitle:\s+\S.+$/m);
    assert.match(diagram, /^\s*accDescr:\s+\S.+$/m);
    assert.doesNotMatch(diagram, /<(?:br|i)>/i);
  }
});

test("the homepage does not preload the Mermaid renderer", () => {
  const home = read("docs/.vitepress/dist/index.html");
  const chunks = readdirSync(new URL("../docs/.vitepress/dist/assets/chunks/", import.meta.url));

  assert.ok(chunks.some((file) => file.startsWith("mermaid.core.")));
  assert.doesNotMatch(home, /mermaid\.core|flowchart-/);
});

test("the shipped Mermaid renderer produces accessible SVGs", async (context) => {
  const { JSDOM } = await import("jsdom");
  const dom = new JSDOM();
  const browserGlobals = [
    ["window", dom.window],
    ["document", dom.window.document],
    ...["Element", "HTMLElement", "SVGElement", "Node", "CSSStyleSheet"].map((name) => [
      name,
      dom.window[name],
    ]),
  ];
  const previousGlobals = new Map(browserGlobals.map(([name]) => [name, globalThis[name]]));
  for (const [name, value] of browserGlobals) globalThis[name] = value;
  dom.window.SVGElement.prototype.getBBox = () => ({ x: 0, y: 0, width: 120, height: 24 });
  dom.window.SVGElement.prototype.getComputedTextLength = () => 120;
  context.after(() => {
    dom.window.close();
    for (const [name, value] of previousGlobals) {
      if (value === undefined) delete globalThis[name];
      else globalThis[name] = value;
    }
  });

  const { default: mermaid } = await import("mermaid");
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: "strict",
    flowchart: { htmlLabels: false },
  });
  for (const [index, diagram] of mermaidDiagrams().entries()) {
    const { svg } = await mermaid.render(`mermaid-test-${index}`, diagram);
    assert.match(svg, /role="graphics-document document"/);
    assert.match(svg, /aria-roledescription="flowchart-v2"/);
    assert.match(svg, /<title[^>]*>.+<\/title>/);
    assert.match(svg, /<desc[^>]*>.+<\/desc>/);
  }
});

test("the landing page speaks the KotoR aural-protocol visual language", () => {
  const home = read("docs/.vitepress/dist/index.html");
  const homeCopy = [
    read("docs/.vitepress/theme/Home.vue"),
    read("docs/.vitepress/theme/DroidPlayground.vue"),
    read("docs/index.md"),
  ].join("\n");

  assert.match(home, /WORDS GO IN\./);
  assert.match(home, /<em>DOOTS COME<br>OUT\.<\/em>/);
  assert.match(home, /MAKE IT DOOT/);
  assert.match(home, /HOW THE DOOTS BEHAVE/);
  assert.match(home, /THE FIELD MANUAL/);
  assert.match(home, /same bytes as last time/);
  assert.match(home, /dootdoot v0\.2\.0 poured/);
  assert.match(home, /After a week, you may even start to think you understand it\./);
  assert.doesNotMatch(
    home,
    /EVERY WORD|SAME PHRASE, SAME CHATTER|sha256 matches golden|No clocks|No drift|minor concern/,
  );
  assert.doesNotMatch(homeCopy, /—|\bNo [^.!?]+[.!?]\s+No\b/i);
  assert.doesNotMatch(home, /A long time ago|Episode XII|class="hyperspace"/);
});

test("the landing page stays pinned to the KotoR reference geometry", () => {
  const home = read("docs/.vitepress/dist/index.html");
  const homeStyles = read("docs/.vitepress/theme/home.css");
  const playgroundStyles = read("docs/.vitepress/theme/playground.css");
  const themeStyles = read("docs/.vitepress/theme/theme.css");
  const config = read("docs/.vitepress/config.mts");

  assert.match(homeStyles, /--protocol-content-width: 1320px/);
  assert.match(homeStyles, /--protocol-page-gutter: 48px/);
  assert.match(homeStyles, /grid-template-columns: 1\.25fr 1fr/);
  assert.match(homeStyles, /gap: 64px/);
  assert.match(homeStyles, /font:\s*700 84px \/ 0\.98 "Chakra Petch"/);
  assert.match(homeStyles, /padding: 12px 22px/);
  assert.match(homeStyles, /padding: 24px 24px 26px/);
  assert.match(homeStyles, /font-size: 12\.5px/);
  assert.match(homeStyles, /line-height: 2\.05/);
  assert.match(homeStyles, /\.terminal-wrap[^{]*\{[^}]*top: -7px/s);
  assert.match(playgroundStyles, /\.console[^{]*\{[^}]*padding: 72px var\(--protocol-gutter\)/s);
  assert.match(playgroundStyles, /\.console-heading[^{]*\{[^}]*margin-bottom: 28px/s);
  assert.match(playgroundStyles, /padding: 34px 36px/);
  assert.match(playgroundStyles, /\.presets[^{]*\{[^}]*gap: 10px/s);
  assert.match(playgroundStyles, /\.presets button[^{]*\{[^}]*padding: 12px 14px/s);
  assert.match(themeStyles, /--vp-nav-height: 54px/);
  assert.match(themeStyles, /\.doot-home \.VPNavBar \.wrapper[^{]*\{[^}]*padding-inline: 48px/s);
  assert.match(themeStyles, /\.doot-home \.VPNavBar \.container[^{]*\{[^}]*max-width: none/s);
  assert.match(themeStyles, /\.VPNavBarMenu[^{]*\{[^}]*gap: 26px/s);
  assert.match(themeStyles, /\.VPNavBarMenuLink[^{]*\{[^}]*padding: 0 !important/s);
  assert.match(
    homeStyles,
    /\.doot-home \.VPContent[^{]*\{[^}]*padding:\s*var\(--vp-nav-height\) 0 0 !important/s,
  );
  assert.match(
    themeStyles,
    /body:has\(\.protocol-home\) \.VPNavBarSearch[^{]*\{[^}]*display: none/s,
  );
  assert.doesNotMatch(home, /LIVE VOICE MODULE/);
  assert.match(themeStyles, /content: "TEXT IN \/\/ DOOTS OUT"/);
  assert.doesNotMatch(themeStyles, /AURAL PROTOCOL|v0\.4/);
  assert.match(home, /rel="icon"/);
  assert.match(config, /text: "GitHub ↗"[^}]*noIcon: true/);
});

test("the browser engine renders arbitrary text through VOICE_V12 WebAssembly", async () => {
  const moduleUrl = new URL("../docs/.vitepress/wasm/dootdoot_core.js", import.meta.url);
  const binaryUrl = new URL("../docs/.vitepress/wasm/dootdoot_core_bg.wasm", import.meta.url);
  const { initSync, render_wav } = await import(moduleUrl);

  initSync({ module: readFileSync(binaryUrl) });
  const first = render_wav("hello, little one");
  const second = render_wav("hello, little one");
  const golden = readFileSync(
    new URL("../dootdoot-core/tests/fixtures/golden/hello_there.wav", import.meta.url),
  );

  assert.ok(first instanceof Uint8Array);
  assert.deepEqual(first.subarray(0, 4), new Uint8Array([82, 73, 70, 70]));
  assert.deepEqual(first, second);
  assert.ok(first.byteLength > 44);
  assert.deepEqual(render_wav("hello there"), new Uint8Array(golden));
});

test("documentation navigation is generated from the source tree", () => {
  for (const group of sidebar.slice(1)) {
    for (const item of group.items) {
      const markdown = read(`docs${item.link}.md`);
      const heading = markdown.match(/^#\s+(.+)$/m)?.[1].replaceAll("`", "");
      assert.equal(item.text, heading, `${item.link} should use its document heading`);
    }
  }

  const validation = sidebar.find((group) => group.text === "Voice validation");
  const versions = validation.items.filter((item) => /VOICE_V\d+/.test(item.text));
  assert.ok(
    versions.findIndex((item) => item.text.includes("VOICE_V2")) <
      versions.findIndex((item) => item.text.includes("VOICE_V10")),
  );
});

test("playground labels match their committed golden renders", () => {
  const corpus = new Map(
    read("dootdoot-core/tests/fixtures/golden_corpus.tsv")
      .trimEnd()
      .split("\n")
      .slice(1)
      .map((line) => line.split("\t")),
  );

  assert.ok(samples.length >= 3);
  for (const { audio, phrase } of samples) {
    const filename = audio.slice(1);
    const label = filename.replace(/\.wav$/, "");
    assert.equal(
      existsSync(new URL(`../dootdoot-core/tests/fixtures/golden/${filename}`, import.meta.url)),
      true,
      `${filename} must be a committed golden fixture`,
    );
    assert.equal(phrase, corpus.get(label), `${filename} must be labelled with its rendered input`);
  }
});

test("the production site is independent of remote font services", () => {
  const assets = new URL("../docs/.vitepress/dist/assets/", import.meta.url);
  const styles = readdirSync(assets)
    .filter((file) => file.endsWith(".css"))
    .map((file) => readFileSync(new URL(file, assets), "utf8"))
    .join("\n");

  assert.doesNotMatch(styles, /fonts\.googleapis\.com|fonts\.gstatic\.com/);
  assert.match(styles, /font-family:Chakra Petch/);
  assert.match(styles, /font-family:IBM Plex Mono/);
  const fonts = readdirSync(assets).filter((file) => file.endsWith(".woff2"));
  assert.ok(fonts.some((file) => file.startsWith("chakra-petch-latin-")));
  assert.ok(fonts.some((file) => file.startsWith("ibm-plex-mono-latin-")));
});

test("the site deploys to the repository GitHub Pages project", () => {
  const workflow = parse(read(".github/workflows/docs.yml"));
  const build = workflow.jobs.build;
  const deploy = workflow.jobs.deploy;
  const actions = [...build.steps, ...deploy.steps]
    .map((step) => step.uses)
    .filter((action) => action !== undefined);

  assert.ok(workflow.on.push.paths.includes("dootdoot-core/tests/fixtures/golden_corpus.tsv"));

  assert.deepEqual(build["runs-on"], ["self-hosted", "macOS", "ARM64"]);
  assert.deepEqual(
    build.steps.filter((step) => step.run).map((step) => step.run),
    [
      "npm install -g npm@11.16.0",
      "npm ci",
      "cargo install wasm-pack --version 0.15.0 --locked",
      "npm run test:docs",
      "command -v gtar || brew install gnu-tar",
      "git clean -ffdx --exclude=target --exclude=node_modules || true",
    ],
  );
  assert.equal(build.permissions.contents, "read");
  assert.equal(deploy.permissions.pages, "write");
  assert.equal(deploy.permissions["id-token"], "write");
  assert.ok(actions.some((action) => action.startsWith("actions/upload-pages-artifact@")));
  assert.ok(actions.some((action) => action.startsWith("actions/deploy-pages@")));
  for (const action of actions) assert.match(action, /@[a-f0-9]{40}$/);

  const home = read("docs/.vitepress/dist/index.html");
  assert.match(home, /href="\/dootdoot\/design"/);
});
