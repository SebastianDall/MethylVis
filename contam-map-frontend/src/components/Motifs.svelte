<script lang="ts">
	let {selectedMotifs = $bindable([]), motifs: allMotifs}  = $props();
	
    function clearSelectedMotifs() {
        selectedMotifs = [];
    }

    function selectAllMotifs() {
        selectedMotifs = allMotifs;
    }
    let selectedMotifSet = $derived(new Set(selectedMotifs));

  function toggleMotif(motif_id: string) {
      if (!selectedMotifs.includes(motif_id)) {
        selectedMotifs = [...selectedMotifs, motif_id];
        console.log("Added. New selectedMotifs:", selectedMotifs);
      } else {
        selectedMotifs = selectedMotifs.filter((c) => c !== motif_id);
        console.log("Removed. New selectedMotifs:", selectedMotifs);
      }
      selectedMotifSet = new Set(selectedMotifs);
  }

</script>


<div class="flex flex-col h-full w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center flex-shrink-0">
    <h2 class="text-xl font-bold">Motifs</h2>
    <button onclick={() => clearSelectedMotifs()} disabled={selectedMotifs.length === 0} class="mx-4 px-2 py-1 rounded-lg {selectedMotifs.length !== 0 ? 'bg-red-200' : 'bg-gray-200'}">Clear</button>
    <button onclick={() => selectAllMotifs()} disabled={selectedMotifs.length === allMotifs.length} class="mx-4 px-2 py-1 rounded-lg {selectedMotifs.length !== allMotifs.length ? 'bg-blue-200' : 'bg-gray-200'}">All</button>
  </div>
    
  {#if allMotifs.length === 0}
    <p class="text-gray-500">No motifs found</p>
  {:else}
 <div class="flex-1 overflow-y-auto min-h-0">
    <ul class="space-y-2">
      {#each allMotifs as motif}
        <li class="border rounded-lg text-sm">
          <button onclick={() => toggleMotif(motif)} class="w-full text-left px-4 py-2 flex items-center {selectedMotifSet.has(motif) ? 'bg-blue-600 text-white hover:bg-blue-400' : 'hover:bg-gray-50'}">
            {motif}
          </button>
        </li>
      {/each}
    </ul>
 </div>
  {/if}
</div>

