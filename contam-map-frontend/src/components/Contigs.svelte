<script lang="ts">
	let { selectedProject, selectedContigs = $bindable([]), contigs}: {
	  selectedProject: string,
	  selectedContigs: string[],
	  contigs: string[],
	} = $props();
	

  let loading = $state(false);
  let error = $state<string | null>(null);

    function clearSelectedContigs() {
        selectedContigs = [];
    }

  function toggleContig(contig_id: string) {
      if (!selectedContigs.includes(contig_id)) {
        selectedContigs = [...selectedContigs, contig_id];
        console.log("Added. New selectedContigs:", selectedContigs);
      } else {
        selectedContigs = selectedContigs.filter((c) => c !== contig_id);
        console.log("Removed. New selectedContigs:", selectedContigs);
      }
  }

  // $effect(() => {
  //   selectedContigs;
  //   contigs = contigs.filter((c) => !selectedContigs.includes(c));
  //   contigs = [...selectedContigs, ...contigs];
    
  // })
  let orderedContigs = $derived.by(() => {
    let selectedSet = new Set(selectedContigs);
    const remaining = contigs.filter((c) => !selectedSet.has(c));
    return [...selectedContigs, ...remaining];
  })
  let selectedSet = $derived(new Set(selectedContigs));

  // function refresh() {
  //   fetchTrigger += 1;
  // }
</script>


<div class="flex flex-col h-full w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center flex-shrink-0">
    <h2 class="text-xl font-bold">Contigs</h2>
    <button onclick={() => clearSelectedContigs()} disabled={selectedContigs.length === 0} class="mx-4 px-2 py-1 rounded-lg {selectedContigs.length !== 0 ? 'bg-red-200' : 'bg-gray-200'}">Clear</button>
  </div>
    
  {#if loading}
    <p>Loading..</p>
  {:else if !selectedProject}
    <p class="text-gray-500">Select a project to view bins</p>
  {:else if error}
    <p class="text-red-500">{error}</p>
  {:else if contigs.length === 0}
    <p class="text-gray-500">No contigs found</p>
  {:else}
 <div class="flex-1 overflow-y-auto min-h-0">
    <ul class="space-y-2">
      {#each orderedContigs as contig}
        <li class="border rounded-lg text-sm">
          <button onclick={() => toggleContig(contig)} class="w-full text-left px-4 py-2 flex items-center {selectedSet.has(contig) ? 'bg-blue-600 text-white hover:bg-blue-400' : 'hover:bg-gray-50'}">
            {contig}
          </button>
        </li>
      {/each}
    </ul>
 </div>
  {/if}
</div>
