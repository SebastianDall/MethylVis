<script lang="ts">
	import type { BinQuality } from "../bindings/BinQuality";
	import type { ErrorResponse } from "../bindings/ErrorResponse";

	let { selectedProject, selectedContigs = $bindable([]) } = $props();
	
	let bins = $state<string[]>([]);
	let selectedBins = $state<string[]>([]);
	let qualities = $state<BinQuality[]>([]);
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

      const queryParams = new URLSearchParams();
      if (qualities.length != 0) {
        qualities.forEach((q) => queryParams.append("quality_filter", q));
      }

      // let queryParams = "";
      // if (qualities.length > 0) {
      //   queryParams = `?quality_filter=${qualities.toString()}`
      // }

      try {
      	const response = await fetch(`/api/projects/${selectedProject}/bins?${queryParams}`);

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

  async function toggleBinContigs(bin: string) {
    console.log("Fetching contigs in bin for: ", bin);
    console.log("Selected contigs are:", selectedContigs);
    error = null;

    try {
    	const response = await fetch(`/api/projects/${selectedProject}/contigs/${bin}`);

    	if (!response.ok) {
    		const error = await response.json() as ErrorResponse;
    		throw new Error(error.message)
    	}

      const contigs: string[] = await response.json();
      console.log("Fethced contigs: ",contigs);

      const allSelected = contigs.every(c => selectedContigs.includes(c));

      if (!allSelected) {
        const newContigs = contigs.filter(c=> !selectedContigs.includes(c));
        console.log("Adding contigs");
        selectedContigs = [...selectedContigs, ...newContigs];
        selectedBins = [...selectedBins, bin];
      } else {
        console.log("Removing contigs");
        selectedContigs = selectedContigs.filter((c) => !contigs.includes(c));
        selectedBins = selectedBins.filter((b) => b !== bin);
        console.log("bins selected: ", selectedBins);
      }
      console.log("selectedContigs after filtering bin", selectedContigs);
    
    } catch (err) {
      error = err instanceof Error ? err.message : "An unknown error has occurred";
    } finally {
      loading=false;
    };
    
  }


  let binQualityMap: {value: BinQuality, label: string}[] = [
    {value: "HQ", label: "HQ"},
    {value: "MQ", label: "MQ"},
    {value: "LQ", label: "LQ"},
  ];
    
  function toggleBinQualities(quality: BinQuality) {
    if (qualities.includes(quality)) {
      qualities = qualities.filter((q) => q !== quality);
    } else {
      qualities = [quality, ...qualities];
    }
  }
  

  
</script>



<div class="flex flex-col w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center">
    <h2 class="text-xl font-bold">Loaded Bins</h2>
    <button onclick={refresh} disabled={loading} class="bg-blue-400 rounded-lg w-20 h-8 text-sm font-bold hover:bg-blue-600 text-white">
      Refresh
    </button>
  </div>
  <div class="flex justify-between w-full items-center">
    {#each binQualityMap as label}
      <button onclick={() => toggleBinQualities(label.value)} class="border rounded-lg font-bold w-12 hover:bg-blue-400 {qualities.includes(label.value) ? 'bg-blue-600 text-white' : 'bg-white'}">{label.label}</button>
    {/each}
  </div>
    
  <div class="flex flex-col w-full space-y-4">
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
      {#each bins.toSorted() as bin}
        <li class="border rounded-lg overflow-hidden">
          <button onclick={() => (toggleBinContigs(bin))} class="w-full text-left px-4 py-2 flex items-center {selectedBins.includes(bin) ? 'bg-blue-600 text-white hover:bg-blue-200' : 'hover:bg-gray-50'}">
            {bin}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  </div>
</div>
