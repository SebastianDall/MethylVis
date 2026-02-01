<script lang="ts"> import { ChevronDown, ChevronUp } from "lucide-svelte";

    let { menuItems, value, onItemSelect = () => {} } = $props();

    let buttonRef: HTMLButtonElement | undefined = $state();
    let dropdownTop = $state(0);
    let dropdownLeft = $state(0);

    
    const dropdownHeight = 100;


  let isOpen = $state(false);

  const handleDropdownClick = () => {
    console.log("Button ref:", buttonRef);
    if (buttonRef) {
        const rect = buttonRef.getBoundingClientRect();

    const spaceBelow = window.innerHeight -  rect.bottom;
    if (spaceBelow < dropdownHeight) {
        dropdownTop = rect.top - dropdownHeight;
    } else {
        dropdownTop = rect.bottom;
    }
    
        dropdownLeft = rect.left;
    }
    isOpen = !isOpen;
  }

  const handleDropdownFocusLoss = ( event: FocusEvent ) => {
    const { relatedTarget, currentTarget } = event;
    if (relatedTarget instanceof HTMLElement && currentTarget instanceof HTMLElement && currentTarget.contains(relatedTarget)) return;

    isOpen = false;
  }

</script>

<div class="dropdown" onfocusout={handleDropdownFocusLoss}>
  <button bind:this={buttonRef} class="btn m-1 border rounded-lg" onclick={handleDropdownClick}>
  {#if isOpen}
    <ChevronDown />
  {:else}
    <ChevronUp />
  {/if}
  </button>
  <ul class="dropdown-content menu p-2 shadow bg-gray-100 border rounded-box w-52 fixed z-50"
    style:visibility={isOpen ? 'visible' : 'hidden'}
    style:top="{dropdownTop}px"
    style:left="{dropdownLeft}px"
    >
    {#each menuItems as item}
  		<li><button onclick={() => onItemSelect(item.value)} class="btn hover:text-slate-900 {value === item.value ? 'text-black' : 'text-slate-500'}">{item.text}</button></li>
  	{/each}
	</ul>
</div>
