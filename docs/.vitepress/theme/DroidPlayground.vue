<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { withBase } from "vitepress";
import { samples } from "./samples.mjs";
import "./playground.css";

const BAR_COUNT = 48;
const PLACEHOLDER = Array.from({ length: BAR_COUNT }, () => 0.08);

const selected = ref(0);
const playing = ref(false);
const progress = ref(0);
const peaksByAudio = ref<Record<string, number[]>>({});
let audio: HTMLAudioElement | undefined;
let sweep = 0;

const sample = computed(() => samples[selected.value]);
const bars = computed(() => peaksByAudio.value[sample.value.audio] ?? PLACEHOLDER);

async function loadPeaks(url: string) {
  if (peaksByAudio.value[url]) return;
  try {
    const response = await fetch(withBase(url));
    const encoded = await response.arrayBuffer();
    const decoded = await new OfflineAudioContext(1, 1, 44100).decodeAudioData(encoded);
    const data = decoded.getChannelData(0);
    const chunk = Math.max(1, Math.floor(data.length / BAR_COUNT));
    const peaks: number[] = [];
    for (let bar = 0; bar < BAR_COUNT; bar += 1) {
      let peak = 0;
      for (let i = bar * chunk; i < (bar + 1) * chunk && i < data.length; i += 8) {
        peak = Math.max(peak, Math.abs(data[i]));
      }
      peaks.push(peak);
    }
    const loudest = Math.max(...peaks, 0.001);
    peaksByAudio.value = {
      ...peaksByAudio.value,
      [url]: peaks.map((peak) => Math.max(0.05, peak / loudest)),
    };
  } catch {
    // Decoding is progressive enhancement; the placeholder bars stay.
  }
}

function trackProgress() {
  if (audio && !audio.paused && audio.duration) {
    progress.value = audio.currentTime / audio.duration;
    sweep = requestAnimationFrame(trackProgress);
  }
}

function play() {
  audio?.pause();
  cancelAnimationFrame(sweep);
  audio = new Audio(withBase(sample.value.audio));
  playing.value = true;
  progress.value = 0;
  audio.addEventListener(
    "ended",
    () => {
      playing.value = false;
      progress.value = 0;
    },
    { once: true },
  );
  audio
    .play()
    .then(trackProgress)
    .catch(() => (playing.value = false));
}

function choose(index: number) {
  selected.value = index;
  play();
}

onMounted(() => {
  for (const item of samples) loadPeaks(item.audio);
});

onBeforeUnmount(() => {
  audio?.pause();
  cancelAnimationFrame(sweep);
});
</script>

<template>
  <section id="signal-console" class="console" aria-labelledby="console-title">
    <div class="console-heading">
      <p class="eyebrow">Incoming transmissions / VOICE_V12 golden renders</p>
      <h2 id="console-title">Hear what<br /><em>meaning</em> sounds like.</h2>
      <p>
        Pick a transmission. Every clip is a golden test fixture rendered by the Rust engine itself,
        not a browser imitation.
      </p>
    </div>
    <div class="console-panel">
      <div class="scope" :class="{ active: playing }" aria-hidden="true">
        <span
          v-for="(peak, index) in bars"
          :key="index"
          :class="{ lit: playing && index / bars.length <= progress }"
          :style="{ '--bar': `${Math.round(peak * 100)}%` }"
        />
        <i>VOICE<br />V12</i>
        <div v-if="playing" class="doot-notes"><b>♪</b><b>doot</b><b>♫</b><b>doot</b><b>♪</b></div>
      </div>
      <div class="readout">
        <span>Input phrase</span><strong>“{{ sample.phrase }}”</strong
        ><small>{{ sample.note }}</small>
      </div>
      <button class="play" type="button" @click="play">
        <span>{{ playing ? "■" : "▶" }}</span
        >{{ playing ? "Transmitting" : "Play transmission" }}
      </button>
      <div class="presets" aria-label="Sample transmissions">
        <button
          v-for="(item, index) in samples"
          :key="item.phrase"
          type="button"
          :class="{ selected: index === selected }"
          @click="choose(index)"
        >
          <span>0{{ index + 1 }}</span
          >{{ item.phrase }}
        </button>
      </div>
      <p class="console-note">
        Want it to say something else? That's the CLI's job: <code>dootdoot "your words"</code>
        <a :href="withBase('/usage')">Get started →</a>
      </p>
    </div>
  </section>
</template>
