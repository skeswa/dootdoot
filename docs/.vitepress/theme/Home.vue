<script setup lang="ts">
import { withBase } from "vitepress";
import DroidPlayground from "./DroidPlayground.vue";
import "./home.css";

const HERO_BARS = [
  7, 43, 21, 20, 43, 35, 38, 33, 38, 11, 34, 5, 26, 24, 27, 5, 24, 41, 15, 22, 12, 35, 9, 26, 32,
  41, 21, 36, 28, 28, 14, 10, 25, 13, 17, 21, 34, 32, 15, 22,
];

const LAWS = [
  {
    number: "01",
    title: "DETERMINISTIC",
    copy: "The same text always produces the same audio, bit-for-bit, across every verified platform. No clocks. No drift.",
  },
  {
    number: "02",
    title: "SEMANTIC",
    copy: "Text with similar meaning sounds similar. Paraphrases rhyme; distant ideas diverge. The dialect can be learned by ear.",
  },
  {
    number: "03",
    title: "DROID BY DESIGN",
    copy: "However you phrase the input, the output is unmistakably the same character. One bounded voice, never recorded.",
  },
];
</script>

<template>
  <main class="protocol-home">
    <section class="protocol-hero" aria-labelledby="hero-title">
      <div class="scanlines" aria-hidden="true" />
      <div class="hero-grid">
        <div class="hero-copy">
          <p class="protocol-kicker"><span>▸</span> SIGNAL ACQUIRED — SEMANTIC AUDIO PROTOCOL</p>
          <h1 id="hero-title">EVERY WORD<br />LEAVES AN<br /><em>ECHO.</em></h1>
          <p class="hero-deck">
            dootdoot reads text and answers in droid—short bursts of warm, warbly chatter.
            Deterministic to the last bit on verified platforms. Meanings that sit close together
            sound close together, so the dialect can be learned by ear.
          </p>
          <div class="hero-actions">
            <a href="#signal-console" class="primary-action"><span>▶</span> OPEN CONSOLE</a>
            <a :href="withBase('/design')" class="text-action">READ THE MANUAL <span>→</span></a>
          </div>
        </div>

        <div class="terminal-wrap" aria-label="Example dootdoot terminal session">
          <div class="terminal-panel">
            <div class="terminal-bar">
              <span>~/signals — zsh</span><b><i /> LIVE</b>
            </div>
            <div class="terminal-body">
              <p><span>$</span> brew install <b>skeswa/tap/dootdoot</b></p>
              <p class="terminal-muted">🍺 dootdoot v0.4.2 poured</p>
              <p class="terminal-gap"><span>$</span> dootdoot <em>"hello, little one"</em></p>
              <div class="terminal-wave" aria-hidden="true">
                <i
                  v-for="(height, index) in HERO_BARS"
                  :key="index"
                  :style="{ height: `${height}px` }"
                />
              </div>
              <p class="terminal-muted">
                <em>doot</em> · 0.9s of chatter · deterministic, bit-for-bit
              </p>
              <p class="terminal-gap">
                <span>$</span> dootdoot <em>"hello, little one"</em> <span>-o</span> echo.wav
              </p>
              <p class="terminal-muted">wrote echo.wav · sha256 matches golden ✓</p>
              <p class="terminal-cursor"><span>$</span> <i /></p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <DroidPlayground />

    <section id="dialect" class="laws" aria-labelledby="laws-title">
      <div class="section-heading">
        <p class="protocol-kicker"><span>▸</span> THE DIALECT</p>
        <h2 id="laws-title">THREE LAWS OF THE VOICE</h2>
      </div>
      <div class="law-grid">
        <article v-for="law in LAWS" :key="law.number">
          <b>{{ law.number }}</b>
          <h3>{{ law.title }}</h3>
          <p>{{ law.copy }}</p>
        </article>
      </div>
    </section>

    <section class="signal-path" aria-labelledby="path-title">
      <div class="section-heading">
        <p class="protocol-kicker"><span>▸</span> SIGNAL PATH</p>
        <h2 id="path-title">MEANING, ROUTED TO SOUND</h2>
      </div>
      <div class="path-grid">
        <div><span>01 / INPUT</span><b>TEXT</b><small>WordPiece units</small></div>
        <i>→</i>
        <div><span>02 / VECTOR</span><b>MEANING</b><small>Four audible axes</small></div>
        <i>→</i>
        <div><span>03 / PLAN</span><b>GESTURE</b><small>Phrase + affect</small></div>
        <i>→</i>
        <div><span>04 / OUTPUT</span><b>VOICE</b><small>Formant synthesis</small></div>
      </div>
      <a :href="withBase('/design#the-core-idea-and-how-the-decisions-hang-together')">
        TRACE THE COMPLETE ARCHITECTURE <span>↗</span>
      </a>
    </section>

    <section id="manual" class="manual" aria-labelledby="manual-title">
      <div class="manual-heading">
        <div class="section-heading">
          <p class="protocol-kicker"><span>▸</span> ARCHIVE ACCESS</p>
          <h2 id="manual-title">THE FIELD MANUAL</h2>
        </div>
        <p>AUTHORITATIVE DOCS — BUILT DIRECTLY FROM THE REPOSITORY</p>
      </div>

      <div class="manual-frame">
        <div class="frame-bar"><i /><i /><i /><span>docs.dootdoot.dev / protocol</span></div>
        <div class="manual-grid">
          <aside>
            <p>PROTOCOL INDEX</p>
            <a :href="withBase('/usage')">01 / OPERATE <span>↗</span></a>
            <a :href="withBase('/design')">02 / UNDERSTAND <span>↗</span></a>
            <a :href="withBase('/spec')">03 / VERIFY <span>↗</span></a>
            <a :href="withBase('/README')">04 / EXPLORE <span>↗</span></a>
          </aside>
          <div class="manual-copy">
            <p class="protocol-kicker">PROTOCOL / THE SOUND LANGUAGE</p>
            <h3>Every phrase maps into a small semantic space.</h3>
            <p>
              Every point in that space renders as a run of chirps, whistles, and warbles. Two
              phrases that mean nearly the same thing land near one another—and sound like it.
            </p>
            <pre><span>$</span> dootdoot "hello, little one" <i>-o</i> echo.wav
<b>doot</b> · rendered locally · VOICE_V12 ✓</pre>
            <div class="field-note">
              <span>◆ FIELD NOTE</span> The browser console and the CLI run the same Rust renderer.
              There is no imitation voice.
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>
