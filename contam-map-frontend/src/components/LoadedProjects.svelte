<script lang="ts">
	import type { ErrorResponse } from "../bindings/ErrorResponse";
	
	let {refreshKey, selectedProject, onProjectSelect = () => {}} = $props();

	let projects = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

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
</script>


<div class="flex flex-col w-full p-8 space-y-4">
  <h2 class="text-xl font-bold">Loaded Projects</h2>
  {#if loading}
    <p>Loading..</p>
  {:else if error}
    <p class="text-red-500">{error}</p>
  {:else if projects.length === 0}
    <p class="text-gray-500">No projects found</p>
  {:else}
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
  {/if}
</div>
