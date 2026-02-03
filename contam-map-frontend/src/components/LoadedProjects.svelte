<script lang="ts">
	import { Save, Loader } from "lucide-svelte";
  import type { ErrorResponse } from "../bindings/ErrorResponse";
	
	let {refreshKey, selectedProject, onProjectSelect = () => {}} = $props();

	let projects = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let saving = $state(false);
  let savingResponse = $state("");

  $effect(() => {
    async function fetchProjects() {
      loading = true;
      error = null;
      refreshKey;

      try {
      	const response = await fetch('/api/projects');

      	if (!response.ok) {
      		const error = await response.json() as ErrorResponse;
      		throw new Error(error.message)
      	}

        projects = await response.json();
      
      } catch (err) {
        error = err instanceof Error ? err.message : "An unknown error has occurred";
      } finally {
        loading=false;
      };
      
    }

    fetchProjects();
  });

    setTimeout(() => {
        savingResponse = "";
        }, 5000);

  async function saveProject() {
    saving = true;

    try {
      const response = await fetch(`/api/projects/save`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(selectedProject)
      });

      if (!response.ok) {
        let error = await response.json();
        throw new Error(error.message);
      }

      savingResponse = await response.json();


    } catch (err) {
      if (err instanceof Error) {
        error = err.message;
      } else {
        error = "Something went wrong during update";
      }
    } finally {
      saving = false;
    }
  }
</script>


<div class="flex flex-col h-full w-full p-8 space-y-4">
  <div class="flex items-center w-full justify-between">
    <h2 class="text-xl font-bold">Loaded Projects</h2>
    <button onclick={saveProject} class="btn group hover:bg-blue-600 p-2 rounded-lg">
      {#if saving}
        <Loader />
      {:else}
        <Save class="group-hover:text-white"/>
      {/if}
    </button>
  </div>

  {#if loading}
    <p>Loading..</p>
  {:else if error}
    <p class="text-red-500">{error}</p>
  {:else if projects.length === 0}
    <p class="text-gray-500">No projects found</p>
  {:else}
    <div class="flex-col flex-1 min-h-0 overflow-y-auto">
    <ul class="space-y-2">
      {#each projects as project}
        <li class="border rounded-lg flex items-center h-10 overflow-hidden">
          <button
          onclick={() => onProjectSelect(project)}
          class="w-full text-left p-4 hover:bg-gray-500 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 {project === selectedProject ? 'bg-blue-400 text-white' : ''}">
          {project}
          </button>
        </li>
      {/each}
    </ul>
    </div>
  {/if}
  {#if savingResponse}
    <p class="bg-green font-xl">{savingResponse}</p>
    {/if}
</div>
