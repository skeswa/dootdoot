import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

test("the documentation site exposes its core reader and playground surfaces", () => {
  assert.equal(existsSync(new URL("../docs/.vitepress/config.mts", import.meta.url)), true);
  assert.match(read("docs/index.md"), /layout: home/);
  assert.match(read("docs/.vitepress/theme/Home.vue"), /DroidPlayground/);
  assert.match(read("docs/.vitepress/theme/DroidPlayground.vue"), /hello_there\.wav/);
});

test("documentation navigation is generated from the source tree", () => {
  const navigation = read("docs/.vitepress/navigation.mts");

  assert.match(navigation, /readdirSync/);
  assert.match(navigation, /reference/);
  assert.match(navigation, /research/);
  assert.match(navigation, /validation/);
});

test("all committed sample paths used by the playground are real voice fixtures", () => {
  const playground = read("docs/.vitepress/theme/DroidPlayground.vue");
  const samples = [...playground.matchAll(/audio: ["']\/([^"']+\.wav)["']/g)];

  assert.ok(samples.length >= 3);
  for (const [, filename] of samples) {
    assert.equal(
      existsSync(new URL(`../dootdoot-core/tests/fixtures/golden/${filename}`, import.meta.url)),
      true,
      `${filename} must be a committed golden fixture`,
    );
  }
});
