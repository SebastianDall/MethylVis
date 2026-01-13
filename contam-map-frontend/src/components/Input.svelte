<script lang="ts">
  let {
    id,
    label,
    value = $bindable(''),
    placeholder= '',
    type = 'text',
    required = false
  } = $props();

</script>

<div class="flex flex-col space-y-4 mb-4">
  <label for={id} class="block text-sm font-medium mb-2">{label}</label>
  <input
    class="w-full border rounded h-[48px] px-4"
    {id}
    {type}
    {placeholder}
    {required}
    bind:value
  />

</div>
