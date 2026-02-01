<script lang="ts">
	import type { ErrorResponse } from "../bindings/ErrorResponse";
	import type { ProjectDetails } from "../bindings/ProjectDetails";
	import Input from "./Input.svelte";

	let { onSuccess = () => {}} = $props();

  let projectId = $state('');
  let methylationPath = $state('');
  let contigBinPath = $state('');
  let binQualityPath = $state('');
  let outputPath = $state('');

  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state(false);

  async function createProject(formData: ProjectDetails) {
  	const response = await fetch('/api/projects/create', {
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

    let formData = {
      "project_id": projectId,
      "methylation_data_path": methylationPath,
      "contig_bin_path": contigBinPath,
      "bin_quality_path": binQualityPath === '' ? null : binQualityPath,
      "output_path": outputPath,
    } as ProjectDetails;
    try {
      await createProject(formData);

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
    label="Project Id"
    id="project-id"
    placeholder="Project Id"
    bind:value={projectId}
  />

  <Input
    label="Methylation Data"
    id="methylation-data"
    placeholder="Methylation Data Path"
    bind:value={methylationPath}
  />
  <Input
    label="Contig Bin"
    id="contig-bin"
    placeholder="Contig bin Data Path"
    bind:value={contigBinPath}
  />
  <Input
    label="Bin Quality Path (optional)"
    id="bin-quality-path"
    placeholder="Bin Quality Data Path"
    bind:value={binQualityPath}
  />
  <Input
    label="Output Path"
    id="output-path"
    placeholder="Project Id"
    bind:value={outputPath}
  />

	<button onclick={handleSubmit} class="w-[200px] bg-blue-400 text-white py-2 rounded-lg hover:bg-blue-600 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors" disabled={!projectId || !methylationPath || !contigBinPath || !outputPath || loading}>
		Create project
	</button>
  </form>

	{#if error}
  	<p>{error}</p>
  {/if}
	{#if success}
  	<p>Project Created</p>
  {/if}

</div>
