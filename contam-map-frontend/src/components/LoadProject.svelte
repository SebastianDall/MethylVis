<script lang="ts">
	import type { ErrorResponse } from "../bindings/ErrorResponse";
	import Input from "./Input.svelte";

	let { onSuccess = () => {}} = $props();

  let projectPath = $state('');

  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state(false);

  async function loadProject(formData: string) {
    console.log(formData);
  	const response = await fetch('/api/projects/load', {
  		method: 'POST',
  		headers: { 'Content-Type': 'application/json'},
  		body: JSON.stringify(formData)
  	});

  	if (!response.ok) {
  		const error = await response.json() as ErrorResponse;
  		throw new Error(error.message)
  	}

  	return response
  }


  async function handleSubmit() {
    loading = true;
    error = null;
    success = false;

    try {
      await loadProject(projectPath);

      success = true;
      onSuccess();
      // projectId = '';
      // methylationPath = '';
      // contigBinPath = '';
      // outputPath = '';

      
    } catch (err) {
      error = err instanceof Error ? err.message : "An unknown error has occurred";
    } finally {
      loading=false;
    };
  } 

</script>


<div class="flex flex-col w-full p-8 space-y-4">
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <Input
    label="Project Path"
    id="project-id"
    placeholder="Project Id"
    bind:value={projectPath}
  />


	<button onclick={handleSubmit} class="w-[200px] bg-blue-400 text-white py-2 rounded-lg hover:bg-blue-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors" disabled={!projectPath || loading}>
	  Load Project
	</button>
  </form>

	{#if error}
  	<p>{error}</p>
  {/if}

</div>
