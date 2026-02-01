<script lang="ts">
  import { onMount } from "svelte";
	import type { ErrorResponse } from "../bindings/ErrorResponse";
	import type { HeatmapData } from "../bindings/HeatmapData";
	import type { MethDataFilters } from "../bindings/MethDataFilters";
	import Dropdown from "./Dropdown.svelte";
	import { Loader, Send } from "lucide-svelte";
	import type { Assignment } from "../bindings/Assignment";
	import type { ContigSelection } from "../bindings/ContigSelection";
	import type { ContigId } from "../bindings/ContigId";
	import type { ContigAssignment } from "../bindings/ContigAssignment";
	
  let { selectedProject, selectedContigs, selectedBin } = $props();
  let loading = $state(false);
  let plotting = $state(false);
  let error = $state("");
  let saveError = $state("");
  let sending = $state(false);

  let minNMotifObs = $state<number | null>(null);
  let minVariance = $state<number | null>(null);
  let minCoverage = $state<number | null>(null);

  let binName = $state("");
  
  let heatmapData = $state<HeatmapData | null>(null);
  let plotDiv: HTMLDivElement;
  let Plotly: any;

  onMount(async () => {
    Plotly = await import("plotly.js-dist-min");
  });

  const assignments = [
    {value: "Clean" as Assignment, text: "✅ Clean"},
    {value: "Contamination" as Assignment, text: "⚠️ Contamination"},
    {value: "Ambiguous" as Assignment, text: "❓ Ambiguous"},
  ];
  const assignmentLabels: Record<Assignment, string> = {
    "None": "➖ None",
    "Clean": "✅ Clean",
    "Contamination": "⚠️ Contamination",
    "Ambiguous": "❓ Ambiguous"
  };


function updateAssignment(contigId: string, assignment: Assignment) {
    console.log("I was clicked", contigId, assignment);
    if (!heatmapData) return;
    if (!heatmapData.metadata) {
        alert("No metadata instance found.");
        return;
    }


    heatmapData!.metadata[contigId]!.assignment = assignment;
    console.log(heatmapData?.metadata[contigId]?.assignment)
    renderHeatmap();
  }


  function resetFilters() {
    minNMotifObs = null;
    minVariance = null;
    minCoverage = null;
  }

  async function handleUpdate() {
    loading = true;
    error = "";

    console.log("Selected Bin in heatmap: ", selectedBin);
    binName = selectedBin;
    const selection = selectedBin ? { "Bin": selectedBin, } as ContigSelection : { "Contigs": selectedContigs, } as ContigSelection;
    const dataQuery = {
       "selection": selection,
       "min_n_motif_obs": minNMotifObs,
       "min_motif_variance": minVariance,
       "min_coverage": minCoverage,
    } as MethDataFilters;

    console.log(dataQuery);

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
    let updatedBin = {
        bin: binName,
        contigs: [] as ContigAssignment[],
    };

    for (const c of selectedContigs) {
        const assignment = heatmapData.metadata[c].assignment;
        if (!assignment || assignment == "None") {
            saveError = "All contigs must have an assignment";
            return;
        }
        let contigAssignment = {
            contig_id: c as ContigId,
            assignment: assignment,
        } as ContigAssignment;

        updatedBin.contigs.push(contigAssignment);
    }

    saveError = ""; 
    sending = true;


    try {
      const response = await fetch(`/api/projects/${selectedProject}/data/update`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(updatedBin)
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
  

  function setToContamination() {
    if(!heatmapData || !heatmapData.metadata) return;

    Object.values(heatmapData.metadata).forEach(metadata => {
      if (metadata?.assignment === "None") {
        metadata.assignment = "Contamination";
      }
    });
  }
  function setToClean() {
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
  <div class="flex flex-col w-84 h-full space-y-4">
    <div class="flex flex-col align-center items-center bg-white rounded-lg p-4">
      <h3 class="font-bold text-lg">Filter</h3>

      <label class="block mb-4">
        <span class="text-sm font-medium">Min Coverage</span>
        <input
          type="number"
          bind:value={minCoverage}
          placeholder="e.g. 5"
          class="w-full mt-1 px-3 py-2 border rounded"
        />
      </label>
      <label class="block mb-4">
        <span class="text-sm font-medium">Min variance</span>
        <input
          class="w-full mt-1 px-3 py-2 border rounded"
          type="number"
          bind:value={minVariance}
          placeholder="e.g. 0.1"
        />
      </label>
      <label class="block mb-4">
        <span class="text-sm font-medium">Min Number of motifs</span>
        <input
          type="number"
          bind:value={minNMotifObs}
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
        <button class="p-1 hover:bg-gray-300 rounded-lg" onclick={setToClean}>✅</button>
        <button class="p-1 hover:bg-gray-300 rounded-lg" onclick={setToContamination}>⚠️</button>
        <button disabled={!binName} class="group hover:bg-blue-600 p-2 rounded-lg {!binName ? 'bg-red-600' : ''}" onclick={sendAssignments}><Send class="group-hover:text-white" /></button>
      </div>
      <div class="w-full items-center mb-4">
        <input
          type="text"
          placeholder="Override bin name"
          class="border rounded-lg px-2 w-full h-10"
          bind:value={binName}
          >

      </div>

      <div class="flex-1 min-h-0 w-full">
        <div class="h-full overflow-y-auto">
        {#if heatmapData}
            <ul class="space-y-2">
            {#each heatmapData.contigs.toReversed() as contigId}
            {@const metadata = heatmapData.metadata?.[contigId]}
                <li class="border rounded-lg">
                  <div class="flex items-center justify-between space-x-2">
                    <p class="px-2">{contigId} | {assignmentLabels[metadata?.assignment ?? "None"]} | {metadata?.mean_coverage?.toFixed(1) ?? "N/A"}</p>
                    <Dropdown menuItems={assignments} value={metadata?.assignment} onItemSelect={(assignment: Assignment) => updateAssignment(contigId, assignment)}/>
                  </div>
                </li>
              {/each}
            </ul>
        {/if}
        </div>


      </div>

    </div>
  </div>
</div>



