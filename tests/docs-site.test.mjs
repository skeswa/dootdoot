import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import test from "node:test";
import { parse } from "yaml";
import { sidebar } from "../docs/.vitepress/navigation.mjs";
import { samples } from "../docs/.vitepress/theme/samples.mjs";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("the production site exposes its reader and playground", () => {
  const home = read("docs/.vitepress/dist/index.html");

  assert.match(home, /id="signal-console"/);
  assert.match(home, /Hear what/);
  assert.match(home, /placeholder="Type a transmission"/);
  assert.match(home, /Rendered locally by VOICE_V12 WebAssembly/);
  assert.equal(existsSync(new URL("../docs/.vitepress/dist/design.html", import.meta.url)), true);
  for (const { audio } of samples) {
    assert.equal(existsSync(new URL(`../docs/.vitepress/dist${audio}`, import.meta.url)), true);
  }
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
  assert.ok(readdirSync(assets).some((file) => file.endsWith(".woff2")));
});

test("the site deploys to the repository GitHub Pages project", () => {
  const workflow = parse(read(".github/workflows/docs.yml"));
  const build = workflow.jobs.build;
  const deploy = workflow.jobs.deploy;
  const actions = [...build.steps, ...deploy.steps]
    .map((step) => step.uses)
    .filter((action) => action !== undefined);

  assert.ok(workflow.on.push.paths.includes("dootdoot-core/tests/fixtures/golden_corpus.tsv"));

  assert.deepEqual(
    build.steps.filter((step) => step.run).map((step) => step.run),
    ["npm ci", "cargo install wasm-pack --version 0.15.0 --locked", "npm run test:docs"],
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
