<!-- frontend/src/lib/components/simcard/EmptyState.svelte -->
<script>
    import Icon from "@iconify/svelte";

    let {
        icon = "mage:memory-card",
        title = "No Data",
        description = "",
        showRetry = false,
        onRetry = () => {},
    } = $props();
</script>

<div class="flex flex-col items-center justify-center py-12 text-center">
    <Icon {icon} class="w-12 h-12 text-gray-400 mb-4" />
    <h3 class="text-lg font-medium text-gray-500 dark:text-gray-400 mb-2">
        {title}
    </h3>
    {#if description}
        <p class="text-sm text-gray-400 dark:text-gray-500 mb-4">
            {description}
        </p>
    {/if}
    {#if showRetry}
        <button
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            onclick={() => {
                onRetry();
            }}
        >
            Try Again
        </button>
    {/if}
</div>
