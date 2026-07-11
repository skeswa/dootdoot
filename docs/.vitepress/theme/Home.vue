<script setup lang="ts">
import DroidPlayground from "./DroidPlayground.vue";
</script>

<template>
  <main class="home-shell">
    <section class="hero" aria-labelledby="hero-title">
      <div class="hero-orbit" aria-hidden="true">
        <div class="orbit orbit-outer" />
        <div class="orbit orbit-inner" />
        <div class="droid-core"><span /></div>
      </div>
      <div class="hero-copy">
        <p class="eyebrow"><span>Voice contract 12</span><i />Semantic audio system</p>
        <h1 id="hero-title">Text,<br /><em>translated</em><br />into droid.</h1>
        <p class="hero-deck">
          Not speech. Not random beeps. A deterministic sound-language where related ideas develop a
          family resemblance.
        </p>
        <div class="hero-actions">
          <a href="#signal-console" class="primary-action">Open signal console <span>↘</span></a>
          <a href="/design" class="text-action">Read the field manual <span>→</span></a>
        </div>
      </div>
      <div class="hero-index" aria-hidden="true">
        <b>01</b><span>Meaning in<br />motion</span>
      </div>
    </section>

    <DroidPlayground />

    <section class="principles" aria-labelledby="principles-title">
      <header>
        <p class="eyebrow">System properties</p>
        <h2 id="principles-title">A language you can <em>learn by ear.</em></h2>
      </header>
      <article>
        <span>01</span>
        <h3>Same words.<br />Same waveform.</h3>
        <p>
          Every sample is pinned by a versioned voice contract. Type it tomorrow or on another
          verified platform—it comes back bit-for-bit.
        </p>
      </article>
      <article>
        <span>02</span>
        <h3>Meaning leaves<br />a sonic trace.</h3>
        <p>
          Words become four perceptual axes: pitch, vowel, contour, and warble. Semantic neighbors
          inherit audible similarities.
        </p>
      </article>
      <article>
        <span>03</span>
        <h3>One small<br />instrument.</h3>
        <p>
          No cloud model and no tensor runtime. The tokenizer and projected semantic map ship inside
          one compact Rust binary.
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
      <a href="/design#the-core-idea-and-how-the-decisions-hang-together"
        >Trace the complete architecture <span>↗</span></a
      >
    </section>

    <section class="docs-portal">
      <p class="eyebrow">Technical archive / 30+ living documents</p>
      <h2>Go beneath<br />the casing.</h2>
      <div class="portal-links">
        <a href="/usage"><span>01 / Operate</span><b>CLI usage</b><i>↗</i></a>
        <a href="/design"><span>02 / Understand</span><b>Design rationale</b><i>↗</i></a>
        <a href="/spec"><span>03 / Verify</span><b>Requirements</b><i>↗</i></a>
        <a href="/README"><span>04 / Explore</span><b>All documentation</b><i>↗</i></a>
      </div>
    </section>
  </main>
</template>
