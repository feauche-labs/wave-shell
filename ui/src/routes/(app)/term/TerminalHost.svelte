<script lang="ts">
  import { sessions } from '$lib/stores/sessions.svelte';
  import TerminalView from '$lib/terminal/TerminalView.svelte';
  // Wave: split screen — each terminal sits in a frame that becomes a pane inside a workspace.
  import PaneFrame from '$lib/wave/PaneFrame.svelte';
  import SplitOverlay from '$lib/wave/SplitOverlay.svelte';
  import { waveSplit } from '$lib/wave/split.svelte';

  const terms = $derived(sessions.tabs.filter((t) => t.kind === 'terminal'));
</script>

<!-- Every terminal tab stays mounted (its session keeps running); only the active one is shown. -->
{#each terms as tab (tab.id)}
  {@const box = waveSplit.boxOf(tab)}
  <PaneFrame {tab} {box}>
    <TerminalView {tab} active={tab.id === sessions.activeId || !!box} />
  </PaneFrame>
{/each}
<SplitOverlay />
