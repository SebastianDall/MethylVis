<script lang="ts">
  import { onMount } from "svelte";
	import type { ErrorResponse } from "../bindings/ErrorResponse";
	import type { HeatmapData } from "../bindings/HeatmapData";
	import type { MethDataFilters } from "../bindings/MethDataFilters";
	import Dropdown from "./Dropdown.svelte";
	import type { ContigAssignment } from "../bindings/ContigAssignment";
	import { Loader, Send } from "lucide-svelte";
	
  let { selectedProject, selectedContigs } = $props();
  let loading = $state(false);
  let plotting = $state(false);
  let error = $state("");
  let saveError = $state("");
  let sending = $state(false);

  let min_n_motif_obs = $state<number | null>(null);
  let min_var = $state<number | null>(null);
  let min_cov = $state<number | null>(null);
  
  let heatmapData = $state<HeatmapData | null>(null);
  let plotDiv: HTMLDivElement;
  let Plotly: any;

  onMount(async () => {
    Plotly = await import("plotly.js-dist-min");
  });

  const assignments = [
    {value: "Clean" as ContigAssignment, text: "✅ Clean"},
    {value: "Contamination" as ContigAssignment, text: "⚠️ Contamination"},
    {value: "Ambiguous" as ContigAssignment, text: "❓ Ambiguous"},
  ];
  const assignmentLabels: Record<ContigAssignment, string> = {
    "None": "➖ None",
    "Clean": "✅ Clean",
    "Contamination": "⚠️ Contamination",
    "Ambiguous": "❓ Ambiguous"
  };


  function updateAssignment(contigId: string, assignment: ContigAssignment) {
    // if (!heatmapData?.metadata) return;

    heatmapData!.metadata[contigId]!.assignment = assignment;
    console.log(heatmapData?.metadata[contigId]?.assignment)
    renderHeatmap();
  }


  function resetFilters() {
    min_n_motif_obs = null;
    min_var = null;
    min_cov = null;
  }

  async function handleUpdate() {
    loading = true;
    error = "";

    const dataQuery = {
       "contigs": selectedContigs,
       "min_n_motif_obs": min_n_motif_obs,
       "min_motif_variance": min_var,
       "min_coverage": min_cov,
    } as MethDataFilters;

    try {
      const response = await fetch(`/api/projects/${selectedProject}/data/heatmap`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(dataQuery),
      });

      if (!response.ok) {
        const error = await response.json() as ErrorResponse;
        throw new Error(error.message);
      }

      heatmapData = await response.json() as HeatmapData;
      loading = false;
      renderHeatmap();
    } catch (err) {
      if (err instanceof Error) {
        error = err.message;
      } else {
        error = "Something went wrong";
      }
    } finally {
      loading = false;
    }
  }

  function renderHeatmap() {
    if (!heatmapData || !Plotly || !plotDiv) return;
    plotting = true;

    const plotData = [{
      x: heatmapData.motifs,
      y: heatmapData.contigs,
      z: heatmapData.matrix,
      type: "heatmap" as const,
      colorscale: "Viridis",
      hoverongaps: false,
    }];

    const layout = {
      title: 'Methylation Heatmap',
      xaxis: {
         title: "Motifs",
         tickangle: -45,
         automargin: true,
      },
      yaxis: {
         title: "Contigs",
         automargin: true,
      },
    }

    Plotly.newPlot(plotDiv, plotData,layout);
    plotting=false;
  }

  async function sendAssignments() {
    if(!heatmapData || !heatmapData.metadata) return;
    const updatedAssignments = Object.values(heatmapData.metadata);
    saveError = "";
    sending = true;


    try {
      const response = await fetch(`/api/projects/${selectedProject}/data/update`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(updatedAssignments)
      });

      if (!response.ok) {
        let error = await response.json();
        throw new Error(error.message);
      }

      sending = false;
      
    } catch (err) {
      if (err instanceof Error) {
        error = err.message;
      } else {
        error = "Something went wrong during update";
      }
    }
  }
  

  function setCleanToNone() {
    if(!heatmapData || !heatmapData.metadata) return;

    Object.values(heatmapData.metadata).forEach(metadata => {
      if (metadata?.assignment === "None") {
        metadata.assignment = "Clean";
      }
    });
  }
</script>



<div class="flex flex-1 gap-4 h-full mb-6">
  <div class="flex flex-col flex-1 bg-white rounded-lg p-4 overflow-hidden">
    <div class="flex-1 flex-col w-full overflow-auto flex items-center justify-center">
      {#if loading}
        <div class="flex items-center space-x-2 rounded-lg bg-green-600 p-2 h-12">
          <Loader />
          <p>Loading heatmap</p>
        </div>
      {:else if plotting}
          <p>Plotting heatmap</p>

      {:else if error}
          <p class="text-red-500">{error}</p>
      {:else if !heatmapData}
        <p>Select contig and click update heatmap</p>
      {/if}
      <div class="flex flex-1 w-full" bind:this={plotDiv}></div>
    </div>
  </div>
  <div class="flex flex-col w-64 h-full space-y-4">
    <div class="flex flex-col align-center items-center bg-white rounded-lg p-4">
      <h3 class="font-bold text-lg">Filter</h3>

      <label class="block mb-4">
        <span class="text-sm font-medium">Min Coverage</span>
        <input
          type="number"
          bind:value={min_cov}
          placeholder="e.g. 5"
          class="w-full mt-1 px-3 py-2 border rounded"
        />
      </label>
      <label class="block mb-4">
        <span class="text-sm font-medium">Min variance</span>
        <input
          class="w-full mt-1 px-3 py-2 border rounded"
          type="number"
          bind:value={min_var}
          placeholder="e.g. 0.1"
        />
      </label>
      <label class="block mb-4">
        <span class="text-sm font-medium">Min Number of motifs</span>
        <input
          type="number"
          bind:value={min_n_motif_obs}
          placeholder="e.g. 5"
          class="w-full mt-1 px-3 py-2 border rounded"
        />
      </label>

    
      <div class="flex justify-between space-x-2">
        <button onclick={handleUpdate} class="mb-4 px-4 py-2 bg-blue-500 text-white rounded-lg">
          <p class="text-sm">Update heatmap</p>
        </button>
        <button onclick={resetFilters} class="mb-4 px-4 py-2 bg-gray-500 text-white rounded-lg">
          <p class="text-sm">Reset filters</p>
        </button>
      </div>
    </div>
    <div class="flex flex-1 flex-col min-h-0 align-center items-center bg-white rounded-lg p-4">
      <div class="flex w-full justify-between items-center">
        <h3 class="font-bold text-lg">Metadata</h3>
        <button onclick={setCleanToNone}>✅</button>
        <button class="group hover:bg-blue-600 p-2 rounded-lg" onclick={sendAssignments}><Send class="group-hover:text-white" /></button>
      </div>

      <div class="flex-1 min-h-0 w-full align-center items-center overflow-y-auto">
        <ul class="space-y-2">
        {#each heatmapData?.contigs.toReversed() as contigId}
            <li class="border rounded-lg overflow-hidden">
              <div class="flex items-center justify-betwen space-x-2">
                <p>{contigId} | {assignmentLabels[heatmapData!.metadata[contigId]!.assignment]} | {heatmapData?.metadata[contigId]?.mean_coverage.toFixed(1)}</p>
                <Dropdown menuItems={assignments} value={heatmapData?.metadata[contigId]?.assignment} onItemSelect={(assignment: ContigAssignment) => updateAssignment(contigId, assignment)}/>
              </div>
            </li>
          {/each}
        </ul>


      </div>

    </div>
  </div>
</div>



