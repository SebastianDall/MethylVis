<script lang="ts">
	import type { ErrorResponse } from "../bindings/ErrorResponse";

	let { selectedProject, selectedContigs = $bindable([])} = $props();
	

	let contigs = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let fetchTrigger = $state(0);

  $effect(() => {
    fetchTrigger;
    console.log("Effect triggered");
    // clear bins if no project is selected
    if (!selectedProject) {
      contigs = [];
      return;
    }

    async function fetchContigs() {
      console.log("Fetching contigs for: ", selectedProject);
      loading = true;
      error = null;

      try {
      	const response = await fetch(`/api/projects/${selectedProject}/contigs`);

      	if (!response.ok) {
      		const error = await response.json() as ErrorResponse;
      		throw new Error(error.message)
      	}

        contigs = await response.json();
      
      } catch (err) {
        error = err instanceof Error ? err.message : "An unknown error has occurred";
      } finally {
        loading=false;
      };
      
    }

    console.log("About to call fetchContigs");
    fetchContigs();
  });

  function toggleContig(contig_id: string) {
      console.log("toggleContig called with:", contig_id);
      console.log("Current selectedContigs:", selectedContigs);
      console.log("Is selected?", selectedContigs.includes(contig_id));

      if (!selectedContigs.includes(contig_id)) {
        selectedContigs = [...selectedContigs, contig_id];
        console.log("Added. New selectedContigs:", selectedContigs);
      } else {
        selectedContigs = selectedContigs.filter((c) => c !== contig_id);
        console.log("Removed. New selectedContigs:", selectedContigs);
      }
  }

  function refresh() {
    fetchTrigger += 1;
  }
</script>


<div class="flex flex-col w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center">
    <h2 class="text-xl font-bold">Loaded Contigs</h2>
    <button onclick={refresh} disabled={loading} class="bg-blue-400 rounded-lg w-20 h-8 text-sm font-bold hover:bg-blue-600 text-white">
      Refresh
    </button>
  </div>
    
  {#if loading}
    <p>Loading..</p>
  {:else if !selectedProject}
    <p class="text-gray-500">Select a project to view bins</p>
  {:else if error}
    <p class="text-red-500">{error}</p>
  {:else if contigs.length === 0}
    <p class="text-gray-500">No bins found</p>
  {:else}
    <ul class="space-y-2">
      {#each contigs as contig}
        <li class="border rounded-lg overflow-hidden">
          <button onclick={() => toggleContig(contig)} class="w-full text-left px-4 py-2 flex items-center hover:bg-gray-50 {selectedContigs.includes(contig) ? 'bg-blue-400 text-white' : ''}">
            {contig}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
