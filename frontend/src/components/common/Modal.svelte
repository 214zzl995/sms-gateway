<!-- frontend/src/lib/components/common/Modal.svelte -->
<script>
    import { fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    
    let { 
        isOpen = false, 
        onClose = () => {},
        maxWidth = "max-w-4xl",
        class: className = "",
        children
    } = $props();

    function handleKeydown(e) {
        if (e.key === 'Escape') onClose();
    }
</script>

{#if isOpen}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
        class="fixed inset-0 flex items-center justify-center
         bg-zinc-100 dark:bg-zinc-800 bg-opacity-50 dark:bg-opacity-30 z-50 backdrop-blur-md"
        transition:fade={{ duration: 200 }}
        onclick={(e) => {
            if (e.target === e.currentTarget) onClose();
        }}
        onkeydown={handleKeydown}
        role="dialog"
        aria-modal="true"
        tabindex="0"
    >
        <div
            class="bg-zinc-100 dark:bg-zinc-800 rounded-lg shadow-xl w-4/5 {maxWidth} max-h-[90vh] flex flex-col {className}"
            transition:fade={{ delay: 100, duration: 300, easing: cubicOut }}
        >
            {@render children?.()}
        </div>
    </div>
{/if}