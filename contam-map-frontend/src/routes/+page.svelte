<script lang="ts">
	import Bins from "../components/Bins.svelte";
	import Contigs from "../components/Contigs.svelte";
	import Heatmap from "../components/Heatmap.svelte";
	import LoadedProjects from "../components/LoadedProjects.svelte";
	import LoadProject from "../components/LoadProject.svelte";
	import ProjectCreation from "../components/ProjectCreation.svelte";

	let refreshKey = $state(0);
  let selectedProject = $state('');
  let selectedContigs = $state<string[]>([]);

	function onProjectUpdate() {
		refreshKey += 1;
	}

	function onProjectSelect(project_id: string) {
		selectedProject = project_id;
		selectedContigs = [];
		console.log(selectedProject);
	}
</script>


<main class="h-screen bg-gray-300 flex items-center justify-center p-4">
	<div class="flex w-full h-full gap-4">
		<div class="bg-white w-md rounded-lg font-bold mb-6 text-center">
			<h1>Projects</h1>
			<ProjectCreation onSuccess={onProjectUpdate} />
			<LoadProject onSuccess={onProjectUpdate} />
			<div class="flex-1 overflow-auto-y">
				<LoadedProjects {refreshKey} {selectedProject} {onProjectSelect}/>
			</div>
		</div>
		<Heatmap {selectedProject} {selectedContigs} />

		<div class="flex flex-col h-full items-center justify-center">
			<div class="flex flex-1 card w-64 bg-white rounded-lg mb-6 overflow-y-auto">
				<Bins {selectedProject} bind:selectedContigs />
			</div>
			<div class="flex flex-1 card w-64 bg-white rounded-lg mb-6 overflow-y-auto">
				<Contigs {selectedProject} bind:selectedContigs />
			</div>
		</div>

	</div>
</main>
