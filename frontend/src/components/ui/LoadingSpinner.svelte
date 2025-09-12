<script>
  import { fade } from "svelte/transition";

  let {
    show = false,
    duration = 150,
    size = "w-5 h-5"
  } = $props();
</script>

{#if show}
  <div
    class="h-full flex justify-center items-center absolute inset-0 z-9"
    transition:fade={{ duration }}
  >
    <div
      class="inline-block animate-spin rounded-full border-2 border-t-gray-800 border-gray-300 {size}"
    ></div>
  </div>
{/if}