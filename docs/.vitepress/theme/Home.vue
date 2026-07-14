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
    title: "RELIABLY WEIRD",
    copy: "A phrase always produces the same WAV, down to the last bit, on macOS and Linux. Run it tomorrow and the droid remembers its lines.",
  },
  {
    number: "02",
    title: "MEANING HAS AN ACCENT",
    copy: "Similar ideas get similar pitch, vowel color, and warble. Listen long enough and the apparent nonsense starts to sound familiar.",
  },
  {
    number: "03",
    title: "BORN SYNTHETIC",
    copy: "The same droid voice synthesizes every sound from scratch. It never reaches for a folder of prerecorded beeps.",
  },
];
</script>

<template>
  <main class="protocol-home">
    <section class="protocol-hero" aria-labelledby="hero-title">
      <div class="scanlines" aria-hidden="true" />
      <div class="hero-grid">
        <div class="hero-copy">
          <p class="protocol-kicker"><span>▸</span> INPUT READY / DROID LISTENING</p>
          <h1 id="hero-title">
            WORDS GO IN.<br /><em>DOOTS COME<br />OUT.</em>
          </h1>
          <p class="hero-deck">
            dootdoot turns whatever you type into warm, warbly droid chatter. Type the same phrase
            twice and it makes the same WAV, byte for byte, on macOS and Linux. Similar meanings
            keep a family resemblance. After a week, you may even start to think you understand it.
          </p>
          <div class="hero-actions">
            <a href="#signal-console" class="primary-action"><span>▶</span> MAKE IT DOOT</a>
            <a :href="withBase('/design')" class="text-action">HOW IT WORKS <span>→</span></a>
          </div>
        </div>

        <div class="terminal-wrap" aria-label="Example dootdoot terminal session">
          <div class="terminal-panel">
            <div class="terminal-bar">
              <span>~/doots / zsh</span><b><i /> LIVE</b>
            </div>
            <div class="terminal-body">
              <p><span>$</span> brew install <b>skeswa/tap/dootdoot</b></p>
              <p class="terminal-muted">🍺 dootdoot v0.2.0 poured</p>
              <p class="terminal-gap"><span>$</span> dootdoot <em>"hello, little one"</em></p>
              <div class="terminal-wave" aria-hidden="true">
                <i
                  v-for="(height, index) in HERO_BARS"
                  :key="index"
                  :style="{ height: `${height}px` }"
                />
              </div>
              <p class="terminal-muted"><em>doot</em> · 0.9s · the droid has opinions</p>
              <p class="terminal-gap">
                <span>$</span> dootdoot <em>"hello, little one"</em> <span>-o</span> echo.wav
              </p>
              <p class="terminal-muted">saved echo.wav · same bytes as last time ✓</p>
              <p class="terminal-cursor"><span>$</span> <i /></p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <DroidPlayground />

    <section id="dialect" class="laws" aria-labelledby="laws-title">
      <div class="section-heading">
        <p class="protocol-kicker"><span>▸</span> WHAT TO EXPECT</p>
        <h2 id="laws-title">HOW THE DOOTS BEHAVE</h2>
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
        <p class="protocol-kicker"><span>▸</span> UNDER THE HOOD</p>
        <h2 id="path-title">HOW WORDS BECOME WARBLE</h2>
      </div>
      <div class="path-grid">
        <div><span>01 / INPUT</span><b>TEXT</b><small>Whatever you typed</small></div>
        <i>→</i>
        <div><span>02 / VECTOR</span><b>MEANING</b><small>Four audible coordinates</small></div>
        <i>→</i>
        <div><span>03 / PLAN</span><b>GESTURE</b><small>Phrasing + mood</small></div>
        <i>→</i>
        <div><span>04 / OUTPUT</span><b>DOOTS</b><small>One synthetic droid</small></div>
      </div>
      <a :href="withBase('/design#the-core-idea-and-how-the-decisions-hang-together')">
        SEE THE OVERENGINEERED DIAGRAM <span>↗</span>
      </a>
    </section>

    <section id="manual" class="manual" aria-labelledby="manual-title">
      <div class="manual-heading">
        <div class="section-heading">
          <p class="protocol-kicker"><span>▸</span> DOCS, SPECS, AND OTHER LIGHT READING</p>
          <h2 id="manual-title">THE FIELD MANUAL</h2>
        </div>
        <p>SOURCE DOCS / BUILT DIRECTLY FROM THE REPOSITORY</p>
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
            <p class="protocol-kicker">UNDER THE HOOD / THE SOUND LANGUAGE</p>
            <h3>The nonsense is extremely organized.</h3>
            <p>
              The renderer places each phrase on a small semantic map, then turns that position into
              pitch, vowel shape, contour, and warble. Nearby meanings land on nearby sounds. Weird,
              but consistent.
            </p>
            <pre><span>$</span> dootdoot "hello, little one" <i>-o</i> echo.wav
<b>doot</b> · rendered locally · same Rust as the CLI ✓</pre>
            <div class="field-note">
              <span>◆ FIELD NOTE</span> The browser console and the CLI run the same Rust renderer.
              Even the WAV bytes match.
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
</template>
