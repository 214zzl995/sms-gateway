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
    <div class="flex border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
        {#each simCards as simCard (simCard.id)}
            <button
                class="px-4 py-3 text-sm font-medium whitespace-nowrap border-b-2 transition-all duration-200 relative
                       {activeSimId === simCard.id 
                         ? 'border-blue-500 text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20' 
                         : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50'}"
                onclick={() => onTabSwitch(simCard.id)}
            >
                <span class="flex items-center space-x-2">
                    <Icon icon="mage:memory-card-fill" class="w-4 h-4" />
                    <span>{getDisplayName(simCard)}</span>
                </span>
                
                {#if activeSimId === simCard.id}
                    <div 
                        class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500 dark:bg-blue-400"
                        transition:slide={{ duration: 200, axis: 'x' }}
                    ></div>
                {/if}
            </button>
        {/each}
    </div>
{/if}