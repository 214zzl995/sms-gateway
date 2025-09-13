<!-- frontend/src/lib/components/simcard/SimCardTabs.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import { slide } from "svelte/transition";
    
    let {
        simCards = [],
        activeSimId = null,
        onTabSwitch = (simId) => {},
        getDisplayName = (sim) => sim.alias || sim.phone_number || `SIM ${sim.id.slice(-8)}`
    } = $props();
</script>

{#if simCards.length > 0}
    <div class="flex border-b border-gray-200 dark:border-zinc-700 overflow-x-auto bg-gray-50 dark:bg-zinc-800/50">
        {#each simCards as simCard (simCard.id)}
            <button
                class="px-6 py-4 text-sm font-medium whitespace-nowrap border-b-2 transition-all duration-200 relative
                       {activeSimId === simCard.id 
                         ? 'border-gray-800 dark:border-gray-200 text-gray-800 dark:text-gray-100 bg-white dark:bg-zinc-900' 
                         : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-zinc-800'}"
                onclick={() => onTabSwitch(simCard.id)}
            >
                <span class="flex items-center gap-2">
                    <Icon icon="carbon:sim-card" class="w-4 h-4" />
                    <span>{getDisplayName(simCard)}</span>
                </span>
                
                {#if activeSimId === simCard.id}
                    <div 
                        class="absolute bottom-0 left-0 right-0 h-0.5 bg-gray-800 dark:bg-gray-200"
                        transition:slide={{ duration: 200, axis: 'x' }}
                    ></div>
                {/if}
            </button>
        {/each}
    </div>
{/if}