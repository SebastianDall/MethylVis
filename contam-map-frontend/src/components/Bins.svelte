<script lang="ts">
	import type { Bin } from "../bindings/Bin";
	import type { BinQuality } from "../bindings/BinQuality";
	import type { ErrorResponse } from "../bindings/ErrorResponse";

	let { selectedProject, selectedContigs = $bindable([]), contigs = $bindable([]), selectedBin = $bindable(null) } = $props();
	
	let bins = $state<Bin[]>([]);
	let filteredBins = $state<Bin[]>([]);
	let selectedBins = $state<Bin[]>([]);
	let qualities = $state<BinQuality[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let fetchTrigger = $state(0);

  $effect(() => {
    fetchTrigger;
    // clear bins if no project is selected
    if (!selectedProject) {
      bins = [];
      return;
    }

    async function fetchBins() {
      console.log("Fetching bins for: ", selectedProject);
      loading = true;
      error = null;

      // const queryParams = new URLSearchParams();
      // if (qualities.length != 0) {
      //   qualities.forEach((q) => queryParams.append("quality_filter", q));
      // }

      try {
      	const response = await fetch(`/api/projects/${selectedProject}/bins`);

      	if (!response.ok) {
      		const error = await response.json() as ErrorResponse;
      		throw new Error(error.message)
      	}

        bins = await response.json();
        filterBinsBasedOnQuality();
        // contigs = bins.flatMap((b) => b.contigs)
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

  let sortedBins = $derived.by(() => {
    let remainingBins = filteredBins.filter((b) => !selectedBins.includes(b));
    return [...selectedBins, ...remainingBins]
  });
  // let sortedBins = $derived(
  //   [...filteredBins].sort((a, b) => a.bin_id.localeCompare(b.bin_id))
  // );

  function filterBinsBasedOnQuality() {
    if (qualities.length === 0) {
      filteredBins = bins;
    } else {
      filteredBins = bins.filter((b) => b.quality !== null && qualities.includes(b.quality));
    }
  }

  function refresh() {
    fetchTrigger += 1;
  }

  async function toggleBinContigs(bin: Bin) {
    error = null;

    
    const allSelected = bin.contig_metadata.every(c=> selectedContigs.includes(c.contig_id));

    if (!allSelected) {
      const newContigs = bin.contig_metadata
        .filter(c=> !selectedContigs.includes(c.contig_id))
        .map(c => c.contig_id);

      console.log("Adding contigs");
      // selectedContigs = selectedContigs.filter(id => !bin.contig_metadata.some(c => c.contig_id === id));
      selectedContigs = [...selectedContigs, ...newContigs];
      selectedBins = [...selectedBins, bin];
    } else {
      console.log("Removing contigs");
      selectedContigs = selectedContigs.filter((id) => !bin.contig_metadata.some(c => c.contig_id === id));
      selectedBins = selectedBins.filter((b) => b.id !== bin.id);
      console.log("bins selected: ", selectedBins);
    }
    contigs = selectedBins.flatMap((b) => b.contig_metadata.map(c=>c.contig_id));

    if (selectedBins.length === 1) {
      selectedBin = bin.id;
    } else {
      selectedBin = null;
    }
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
    filterBinsBasedOnQuality();
  }
  

  
</script>



<div class="flex flex-col h-full w-full p-4 space-y-4">
  <div class="flex justify-between w-full items-center flex-shrink-0">
    <h2 class="text-xl font-bold">Loaded Bins</h2>
    <button onclick={refresh} disabled={loading} class="bg-blue-400 rounded-lg w-20 h-8 text-sm font-bold hover:bg-blue-600 text-white">
      Refresh
    </button>
  </div>
  <div class="flex justify-between w-full items-center flex-shrink-0">
    {#each binQualityMap as label}
      <button onclick={() => toggleBinQualities(label.value)} class="border rounded-lg font-bold w-12 hover:bg-blue-400 {qualities.includes(label.value) ? 'bg-blue-600 text-white' : 'bg-white'}">{label.label}</button>
    {/each}
  </div>
    
  <div class="flex-1 overflow-y-auto min-h-0">
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
      {#each sortedBins as bin}
        <li class="border rounded-lg overflow-hidden">
          <button onclick={() => (toggleBinContigs(bin))} class="w-full text-left px-4 py-2 flex items-center {selectedBins.includes(bin) ? 'bg-blue-600 text-white hover:bg-blue-200' : 'hover:bg-gray-50'}">
            {bin.id} n={bin.contig_metadata.length} | {bin.quality} | {bin.completeness?.toFixed(1)} | {bin.contamination?.toFixed(1)}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  </div>
</div>
