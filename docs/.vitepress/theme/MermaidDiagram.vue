<script setup lang="ts">
import { onMounted, ref } from "vue";
import { loadMermaid } from "./mermaid";

const props = defineProps<{
  diagramId: string;
  source: string;
}>();

const error = ref("");
const loading = ref(true);
const output = ref<HTMLElement>();
const graph = decodeURIComponent(props.source);
const isWide = /^flowchart\s+LR\b/m.test(graph);

onMounted(async () => {
  try {
    await document.fonts?.ready;
    const mermaid = await loadMermaid();
    const { bindFunctions, svg } = await mermaid.render(props.diagramId, graph);
    if (output.value === undefined) return;

    output.value.innerHTML = svg;
    bindFunctions?.(output.value);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : "The diagram could not be rendered.";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <figure class="mermaid-diagram" :class="{ 'mermaid-diagram--wide': isWide }" :aria-busy="loading">
    <div class="mermaid-diagram__canvas">
      <p v-if="loading" class="mermaid-diagram__status">Plotting signal path…</p>
      <div ref="output" class="mermaid-diagram__output" :hidden="loading || error.length > 0"></div>
      <div v-if="!loading && error" class="mermaid-diagram__error" role="alert">
        <p>{{ error }}</p>
        <pre><code>{{ graph }}</code></pre>
      </div>
    </div>
  </figure>
</template>
