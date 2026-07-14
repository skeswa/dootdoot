<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { withBase } from "vitepress";
import { samples } from "./samples.mjs";
import "./playground.css";

const BAR_COUNT = 48;
const PLACEHOLDER = Array.from({ length: BAR_COUNT }, () => 0.08);

const phrase = ref(samples[0].phrase);
const selected = ref(0);
const playing = ref(false);
const progress = ref(0);
const bars = ref(PLACEHOLDER);
const status = ref("Droid on standby");
let audio: HTMLAudioElement | undefined;
let objectUrl: string | undefined;
let renderWav: ((text: string) => Uint8Array) | undefined;
let voiceLoad: Promise<void> | undefined;
let sweep = 0;

// The module weighs megabytes, so it must never compete with the initial page
// load: it fetches after the window load event at idle priority, or sooner if
// the visitor reaches for the console first.
function loadVoice() {
  voiceLoad ??= (async () => {
    status.value = "Warming up VOICE_V12…";
    try {
      const module = await import("../wasm/dootdoot_core.js");
      await module.default();
      renderWav = module.render_wav;
      status.value = "Droid ready";
    } catch {
      voiceLoad = undefined;
      status.value = "Voice module unavailable. Reload to retry.";
    }
  })();
  return voiceLoad;
}

const note = computed(
  () => samples.find((item) => item.phrase === phrase.value)?.note ?? "Fresh from your browser",
);

async function decodePeaks(wav: Uint8Array) {
  try {
    const encoded = wav.buffer.slice(wav.byteOffset, wav.byteOffset + wav.byteLength);
    const decoded = await new OfflineAudioContext(1, 1, 44100).decodeAudioData(encoded);
    const data = decoded.getChannelData(0);
    const chunk = Math.max(1, Math.floor(data.length / BAR_COUNT));
    const peaks: number[] = [];
    for (let bar = 0; bar < BAR_COUNT; bar += 1) {
      let peak = 0;
      for (let index = bar * chunk; index < (bar + 1) * chunk && index < data.length; index += 8) {
        peak = Math.max(peak, Math.abs(data[index]));
      }
      peaks.push(peak);
    }
    const loudest = Math.max(...peaks, 0.001);
    bars.value = peaks.map((peak) => Math.max(0.05, peak / loudest));
  } catch {
    bars.value = PLACEHOLDER;
  }
}

function releaseAudio() {
  audio?.pause();
  audio = undefined;
  cancelAnimationFrame(sweep);
  if (objectUrl) URL.revokeObjectURL(objectUrl);
  objectUrl = undefined;
}

function trackProgress() {
  if (audio && !audio.paused && audio.duration) {
    progress.value = audio.currentTime / audio.duration;
    sweep = requestAnimationFrame(trackProgress);
  }
}

async function play() {
  if (!renderWav) {
    await loadVoice();
    if (!renderWav) return;
  }
  releaseAudio();
  playing.value = false;
  progress.value = 0;
  status.value = "Synthesizing locally…";
  await nextTick();

  try {
    const wav = renderWav(phrase.value);
    void decodePeaks(wav);
    objectUrl = URL.createObjectURL(new Blob([wav], { type: "audio/wav" }));
    audio = new Audio(objectUrl);
    playing.value = true;
    status.value = "Doot in progress";
    audio.addEventListener(
      "ended",
      () => {
        playing.value = false;
        progress.value = 0;
        status.value = "Droid ready";
      },
      { once: true },
    );
    await audio.play();
    trackProgress();
  } catch (error) {
    releaseAudio();
    playing.value = false;
    status.value = error instanceof Error ? error.message : "The droid could not render that one.";
  }
}

async function choose(index: number) {
  selected.value = index;
  phrase.value = samples[index].phrase;
  await play();
}

onMounted(() => {
  const whenIdle = () => {
    if ("requestIdleCallback" in window) {
      requestIdleCallback(() => void loadVoice(), { timeout: 4000 });
    } else {
      setTimeout(() => void loadVoice(), 1200);
    }
  };
  if (document.readyState === "complete") {
    whenIdle();
  } else {
    window.addEventListener("load", whenIdle, { once: true });
  }
});

onBeforeUnmount(releaseAudio);
</script>

<template>
  <section id="signal-console" class="console" aria-labelledby="console-title">
    <div class="console-inner">
      <header class="console-heading">
        <h2 id="console-title">MAKE IT DOOT</h2>
        <p>THE SAME RUST VOICE RUNS HERE AND IN THE CLI. YOUR BROWSER IS A DROID NOW.</p>
      </header>

      <div class="console-panel">
        <div class="transmission-compose">
          <form class="readout" @submit.prevent="play">
            <label for="transmission">GIVE THE DROID A SENTENCE</label>
            <input
              id="transmission"
              v-model="phrase"
              type="text"
              placeholder="Type something. Anything."
              autocomplete="off"
              spellcheck="false"
              @input="selected = -1"
            />
            <small>{{ note }} · press enter or poke the button</small>
          </form>

          <div class="console-action-row">
            <button class="play" type="button" @click="play">
              <span>{{ playing ? "■" : "▶" }}</span
              >{{ playing ? "DOOTING" : "MAKE IT DOOT" }}
            </button>
            <div class="scope" :class="{ active: playing }" aria-hidden="true">
              <span
                v-for="(peak, index) in bars"
                :key="index"
                :class="{ lit: playing && index / bars.length <= progress }"
                :style="{ '--bar': `${Math.round(peak * 100)}%` }"
              />
              <i>V12</i>
            </div>
          </div>
          <p class="console-status" aria-live="polite"><i />{{ status }}</p>
        </div>

        <aside class="transmission-presets">
          <p>THINGS TO SAY</p>
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
            CLI: <code>brew install skeswa/tap/dootdoot</code> /
            <a :href="withBase('/usage')">BUILD IT YOURSELF ↗</a>
          </p>
        </aside>
      </div>
    </div>
  </section>
</template>
