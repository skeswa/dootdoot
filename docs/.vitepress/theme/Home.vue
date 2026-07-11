<script setup lang="ts">
import { withBase } from "vitepress";
import DroidPlayground from "./DroidPlayground.vue";
import "./home.css";

const HYPER_HUES = ["#cdd6ff", "#b9a8ff", "#e8ecff", "#9fb4ff", "#d9c9ff"];

const CRAWL_PARAS = [
  `It is a period of silent terminals. Builds finish, tests pass, pipes burst, and the
   shell says nothing at all.`,
  `From a hidden repository, engineers have taught the command line to speak droid. Every
   word you type becomes a run of chirps, whistles, and warbles, identical to the last bit
   every single time.`,
  `Words that mean similar things sound alike. Listen for a week and you will know a
   passing test from a burning one with your eyes closed.`,
  `Armed with one small Rust binary and no cloud at all, the little voice now rides with
   every terminal that will have it. May your builds be green and your doots be many….`,
];

function hyperStyle(n: number) {
  return {
    "--a": `${(n * 137.508) % 360}deg`,
    "--o": `${0.25 + ((n * 29) % 50) / 100}`,
    animationDuration: `${1.7 + (n % 7) * 0.33}s`,
    animationDelay: `${-((n * 0.37) % 3.2)}s`,
    color: HYPER_HUES[n % HYPER_HUES.length],
  };
}
</script>

<template>
  <main class="home-shell">
    <section class="hero" aria-labelledby="hero-title">
      <div class="stars" aria-hidden="true" />
      <div class="hyperspace" aria-hidden="true">
        <span v-for="n in 70" :key="n" :style="hyperStyle(n)" />
      </div>
      <div class="hero-copy">
        <p class="crawl-line">A long time ago, in a terminal far, far away….</p>
        <p class="eyebrow"><span>Voice contract 12</span><i />Semantic audio system</p>
        <h1 id="hero-title">Text,<br /><em>translated</em><br />into droid.</h1>
        <p class="hero-deck">
          Type anything and your terminal answers in droid: chirps, whistles, and warbles that come
          back bit-for-bit identical every time. Words with similar meanings sound alike, so you can
          learn the dialect by ear.
        </p>
        <div class="hero-actions">
          <a href="#signal-console" class="primary-action">Open signal console <span>↘</span></a>
          <a :href="withBase('/design')" class="text-action"
            >Read the field manual <span>→</span></a
          >
        </div>
      </div>
      <div class="crawl-stage" aria-hidden="true">
        <div class="crawl-track">
          <div v-for="copy in 2" :key="copy" class="crawl-copy">
            <p class="crawl-ep">Episode XII</p>
            <p class="crawl-title">A new doot</p>
            <p v-for="(para, index) in CRAWL_PARAS" :key="index">{{ para }}</p>
          </div>
        </div>
      </div>
      <div class="film" aria-hidden="true" />
    </section>

    <DroidPlayground />

    <section class="principles" aria-labelledby="principles-title">
      <header>
        <p class="eyebrow">Droid physics</p>
        <h2 id="principles-title">A language you can <em>learn by ear.</em></h2>
      </header>
      <article>
        <span>01</span>
        <h3>Same words.<br />Same waveform.</h3>
        <p>
          Every sample is pinned by a versioned voice contract. Type it tomorrow, or on another
          verified platform, and it comes back bit-for-bit.
        </p>
      </article>
      <article>
        <span>02</span>
        <h3>Meaning leaves<br />a sonic trace.</h3>
        <p>
          Words land on four audible axes: pitch, vowel, contour, and warble. Neighbors in meaning
          become neighbors in sound.
        </p>
      </article>
      <article>
        <span>03</span>
        <h3>One small<br />instrument.</h3>
        <p>
          No cloud, no tensor runtime. The tokenizer and the projected semantic map ship inside one
          small Rust binary.
        </p>
      </article>
    </section>

    <section class="pipeline" aria-label="Signal pipeline">
      <p class="eyebrow">Signal path / deterministic end to end</p>
      <div class="pipeline-track">
        <div><small>01</small><b>Text</b><span>WordPiece units</span></div>
        <i>→</i>
        <div><small>02</small><b>Meaning</b><span>4 semantic axes</span></div>
        <i>→</i>
        <div><small>03</small><b>Performance</b><span>Phrase + affect</span></div>
        <i>→</i>
        <div><small>04</small><b>Voice</b><span>Formant synthesis</span></div>
        <i>→</i>
        <div><small>05</small><b>Wave</b><span>44.1 kHz mono</span></div>
      </div>
      <a :href="withBase('/design#the-core-idea-and-how-the-decisions-hang-together')"
        >Trace the complete architecture <span>↗</span></a
      >
    </section>

    <section class="docs-portal">
      <p class="eyebrow">The archives / 30+ schematics and field manuals</p>
      <h2>Go beneath<br />the casing.</h2>
      <div class="portal-links">
        <a :href="withBase('/usage')"><span>01 / Operate</span><b>CLI usage</b><i>↗</i></a>
        <a :href="withBase('/design')"
          ><span>02 / Understand</span><b>Design rationale</b><i>↗</i></a
        >
        <a :href="withBase('/spec')"><span>03 / Verify</span><b>Requirements</b><i>↗</i></a>
        <a :href="withBase('/README')"><span>04 / Explore</span><b>All documentation</b><i>↗</i></a>
      </div>
    </section>
  </main>
</template>
