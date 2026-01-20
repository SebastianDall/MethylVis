<script lang="ts">
	import { ChevronDown, ChevronUp } from "lucide-svelte";

	let { menuItems, value, onItemSelect = () => {} } = $props();

  let isOpen = $state(false);

  const handleDropdownClick = () => {
    isOpen = !isOpen;
  }

  const handleDropdownFocusLoss = ( event: FocusEvent ) => {
    const { relatedTarget, currentTarget } = event;
    if (relatedTarget instanceof HTMLElement && currentTarget instanceof HTMLElement && currentTarget.contains(relatedTarget)) return;

    isOpen = false;
  }

</script>

<div class="dropdown" onfocusout={handleDropdownFocusLoss}>
  <button class="btn m-1 border rounded-lg" onclick={handleDropdownClick}>
  {#if isOpen}
    <ChevronDown />
  {:else}
    <ChevronUp />
  {/if}
  </button>
  <ul class="dropdown-content menu p-2 shadow bg-gray-100 border rounded-box w-52 absolute z-50" style:visibility={isOpen ? 'visible' : 'hidden'}>
    {#each menuItems as item}
  		<li><button onclick={() => onItemSelect(item.value)} class="btn hover:text-slate-900 {value === item.value ? 'text-black' : 'text-slate-500'}">{item.text}</button></li>
  	{/each}
	</ul>
</div>
