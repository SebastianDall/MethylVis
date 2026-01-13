<script lang="ts">
	import type { ErrorResponse } from "../bindings/ErrorResponse";

	let { selectedProject } = $props();
	

	let bins = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let fetchTrigger = $state(0);

  $effect(() => {
    fetchTrigger;
    console.log("Effect triggered");
    // clear bins if no project is selected
    if (!selectedProject) {
      bins = [];
      return;
    }

    async function fetchBins() {
      console.log("Fetching bins for: ", selectedProject);
      loading = true;
      error = null;

      try {
      	const response = await fetch(`/api/projects/${selectedProject}/bins`);

      	if (!response.ok) {
      		const error = await response.json() as ErrorResponse;
      		throw new Error(error.message)
      	}

        bins = await response.json();
        console.log(bins);
      
      } catch (err) {
        error = err instanceof Error ? err.message : "An unknown error has occurred";
      } finally {
        loading=false;
      };
      
    }

    console.log("About to call fetchBins");
    fetchBins();
  });

  function refresh() {
    fetchTrigger += 1;
  }
</script>


<div class="flex flex-col w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center">
    <h2 class="text-xl font-bold">Loaded Bins</h2>
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
  {:else if bins.length === 0}
    <p class="text-gray-500">No bins found</p>
  {:else}
    <ul class="space-y-2">
      {#each bins as bin}
        <li class="border rounded-lg overflow-hidden">
          <button class="w-full text-left px-4 py-2 flex items-center hover:bg-gray-50">
            {bin}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
