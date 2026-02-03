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
	
  let { selectedProject, selectedContigs = $bindable([]), selectedBin = $bindable(), allMotifs = $bindable([]), selectedMotifs = $bindable([]) } = $props();
  let loading = $state(false);
  let plotting = $state(false);
  let error = $state("");
  let saveError = $state("");
  let sending = $state(false);

  let minNMotifObs = $state<number | null>(null);
  let minVariance = $state<number | null>(null);
  let minCoverage = $state<number | null>(null);
  let minMeth = $state<number | null>(null);

  let binName = $state("");
  
  let fullHeatmapData = $state<HeatmapData | null>(null);
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
    if (!fullHeatmapData) return;
    if (!fullHeatmapData.metadata) {
        alert("No metadata instance found.");
        return;
    }


    fullHeatmapData!.metadata[contigId]!.assignment = assignment;
    console.log(fullHeatmapData?.metadata[contigId]?.assignment)
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
       "min_methylation_value": minMeth,
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

      fullHeatmapData = await response.json() as HeatmapData;
      loading = false;
      selectedMotifs = fullHeatmapData.motifs;
      allMotifs = selectedMotifs;
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

  let filteredHeatmapData = $derived.by(() => {
        if (!fullHeatmapData || !selectedMotifs) return null;

        const selectedMotifsSet = new Set(selectedMotifs);
        const selectedIndices = fullHeatmapData.motifs.map((motif, index) => selectedMotifsSet.has(motif) ? index : -1).filter(i => i !== -1);

        const filteredMotifs = selectedIndices.map(i => fullHeatmapData.motifs[i]);
        const filteredMatrix = fullHeatmapData.matrix.map(row => selectedIndices.map(i => row[i]));

        return {
            ...fullHeatmapData,
            motifs: filteredMotifs,
            matrix: filteredMatrix,
        };
      });

  $effect(() => {
      selectedMotifs;
      if (filteredHeatmapData) {
          renderHeatmap();
      }
  });

  function renderHeatmap() {
    if (!filteredHeatmapData || !Plotly || !plotDiv) return;
    plotting = true;

    const plotData = [{
  x: filteredHeatmapData.motifs,
  y: filteredHeatmapData.contigs,
  z: filteredHeatmapData.matrix,
  type: "heatmap" as const,
  colorscale: "Magma",
  // colorscale: [
  //   [0, 'rgb(255, 255, 255)'],     // 0 = white
  //   [0.5, 'rgb(100, 100, 255)'],   // 0.5 = light blue
  //   [1, 'rgb(0, 0, 139)']          // 1 = dark blue
  // ],
  zmin: 0,
  zmax: 1,
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

Plotly.react(plotDiv, plotData, layout);
plotting=false;
}

async function sendAssignments() {
if(!fullHeatmapData || !fullHeatmapData.metadata) return;
let updatedBin = {
    bin: binName,
    contigs: [] as ContigAssignment[],
};

for (const c of selectedContigs) {
    const assignment = fullHeatmapData.metadata[c].assignment;
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
    if(!fullHeatmapData || !fullHeatmapData.metadata) return;

    Object.values(fullHeatmapData.metadata).forEach(metadata => {
      if (metadata?.assignment === "None") {
        metadata.assignment = "Contamination";
      }
    });
  }
  function setToClean() {
    if(!fullHeatmapData || !fullHeatmapData.metadata) return;

    Object.values(fullHeatmapData.metadata).forEach(metadata => {
      if (metadata?.assignment === "None") {
        metadata.assignment = "Clean";
      }
    });
  }
    function clearSelectedContigs() {
        selectedContigs = [];
    }
    function selectAllContigs() {
        if (!fullHeatmapData) return;
        selectedContigs = fullHeatmapData.contigs;
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

    $effect(() => {
        const currentSelectedContigs = selectedContigs;
        const currentFullHeatmapData = fullHeatmapData;
        const currentSelectedBin = selectedBin;
        if (!currentFullHeatmapData || !currentSelectedBin) return;

        const allContigsInBin = currentFullHeatmapData.contigs;
        const allSelected = allContigsInBin.every(contig => currentSelectedContigs.includes(contig));
        
        binName = allSelected ? currentSelectedBin : "";

        });
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
      {:else if !fullHeatmapData}
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
      <label class="block mb-4">
        <span class="text-sm font-medium">Minimum motif methylation value</span>
        <input
          type="number"
          bind:value={minMeth}
          placeholder="e.g. 0.1"
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
      <div class="flex w-full justify-between items-center mb-2">
        <h3 class="font-bold text-lg">Metadata</h3>
        <button class="p-1 hover:bg-gray-300 rounded-lg" onclick={clearSelectedContigs}>Clear</button>
        <button class="p-1 hover:bg-gray-300 rounded-lg" onclick={selectAllContigs}>All</button>
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
        {#if fullHeatmapData}
            <ul class="space-y-2">
            {#each fullHeatmapData.contigs.toReversed() as contigId}
            {@const metadata = fullHeatmapData.metadata?.[contigId]}
                <li class="border rounded-lg">
                  <div onclick={() => toggleContig(contigId)} class="flex items-center justify-between space-x-2 {!selectedContigs.includes(contigId) ? 'bg-gray-300' : ''}">
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



