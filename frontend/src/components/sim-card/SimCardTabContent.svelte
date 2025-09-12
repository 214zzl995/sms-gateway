<!-- frontend/src/lib/components/simcard/SimCardTabContent.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import { fly, fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { onDestroy } from "svelte";
    import SimCardBasicInfo from "./SimCardBasicInfo.svelte";
    import SimCardTechnicalInfo from "./SimCardTechnicalInfo.svelte";
    import SimCardBasicInfoSkeleton from "./SimCardBasicInfoSkeleton.svelte";
    import SimCardTechnicalInfoSkeleton from "./SimCardTechnicalInfoSkeleton.svelte";
    import EmptyState from "./EmptyState.svelte";

    let {
        simCard = null,
        simInfo = null,
        isLoading = false,
        onRefresh = () => {},
        onUpdatePhone = async (phone) => true,
        onUpdateAlias = async (alias) => true,
        getDisplayName = (sim) =>
            sim.alias || sim.phone_number || `SIM ${sim.id.slice(-8)}`,
    } = $props();

    let isRefreshing = $state(false);
    let minLoadingTimer = null;
    let skeletonStartTime = $state(null);
    let showSkeleton = $state(false);

    function handleRefresh() {
        if (isRefreshing) return;
        
        isRefreshing = true;
        const startTime = Date.now();
        const minLoadingDuration = 1000; // 1 second minimum
        
        // Clear any existing timer
        if (minLoadingTimer) {
            clearTimeout(minLoadingTimer);
        }
        
        // Start the actual refresh
        const refreshPromise = onRefresh();
        
        // Ensure minimum loading time
        minLoadingTimer = setTimeout(() => {
            isRefreshing = false;
        }, minLoadingDuration);
        
        // If refresh completes after minimum time, stop immediately
        Promise.resolve(refreshPromise).then(() => {
            const elapsed = Date.now() - startTime;
            if (elapsed >= minLoadingDuration) {
                if (minLoadingTimer) {
                    clearTimeout(minLoadingTimer);
                }
                isRefreshing = false;
            }
        }).catch(() => {
            const elapsed = Date.now() - startTime;
            if (elapsed >= minLoadingDuration) {
                if (minLoadingTimer) {
                    clearTimeout(minLoadingTimer);
                }
                isRefreshing = false;
            }
        });
    }

    // Manage skeleton display with minimum 1 second duration
    $effect(() => {
        if (isLoading && !simInfo) {
            // Starting to load - show skeleton and record time
            showSkeleton = true;
            skeletonStartTime = Date.now();
        } else if (simInfo && skeletonStartTime) {
            // Data loaded - check if minimum time has passed
            const elapsedTime = Date.now() - skeletonStartTime;
            const minSkeletonDuration = 1000; // 1 second minimum
            
            if (elapsedTime < minSkeletonDuration) {
                // Need to wait more time
                setTimeout(() => {
                    showSkeleton = false;
                    skeletonStartTime = null;
                }, minSkeletonDuration - elapsedTime);
            } else {
                // Minimum time already passed
                showSkeleton = false;
                skeletonStartTime = null;
            }
        }
    });

    // Cleanup timers on component destroy
    onDestroy(() => {
        if (minLoadingTimer) {
            clearTimeout(minLoadingTimer);
        }
    });
</script>

{#if simCard}
    <div
        in:fly={{ x: 20, duration: 250, easing: cubicOut }}
        out:fade={{ duration: 150 }}
    >
        <!-- SIM Card Header - Always visible -->
        <div class="flex justify-between items-center mb-6">
            <div class="flex items-center space-x-3">
                <div class="p-2 bg-blue-100 dark:bg-blue-900 rounded-lg">
                    <Icon
                        icon="mage:memory-card-fill"
                        class="w-6 h-6 text-blue-600 dark:text-blue-400"
                    />
                </div>
                <div>
                    <h3
                        class="text-lg font-semibold text-gray-800 dark:text-gray-200"
                    >
                        {getDisplayName(simCard)}
                    </h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        IMSI: {simCard.imsi}
                    </p>
                </div>
            </div>

            <div class="flex items-center space-x-2">
                <button
                    class="p-2 text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300
                           hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-colors
                           {isRefreshing ? 'cursor-not-allowed opacity-75' : ''}"
                    onclick={handleRefresh}
                    title="Refresh SIM info"
                    disabled={isRefreshing}
                >
                    <Icon
                        icon="mage:refresh"
                        width="18"
                        height="18"
                        class={isRefreshing ? "animate-spin" : ""}
                    />
                </button>
                {#if simInfo?.model_info?.model}
                    <span
                        class="px-3 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 text-sm font-medium rounded-full"
                    >
                        {simInfo.model_info.model}
                    </span>
                {/if}
            </div>
        </div>
        
        <!-- SIM Information Content -->
        {#if showSkeleton || (isLoading && !simInfo) || isRefreshing}
            <!-- Skeleton loading state -->
            <div 
                class="grid grid-cols-1 md:grid-cols-2 gap-6 h-[22rem]"
                in:fade={{ duration: 200 }}
            >
                <SimCardBasicInfoSkeleton />
                <SimCardTechnicalInfoSkeleton />
            </div>
        {:else if simInfo}
            <!-- Actual content -->
            <div
                class="grid grid-cols-1 md:grid-cols-2 gap-6 h-[22rem]"
                in:fade={{ delay: 100, duration: 300 }}
            >
                <SimCardBasicInfo
                    {simCard}
                    {simInfo}
                    {onUpdatePhone}
                    {onUpdateAlias}
                />
                <SimCardTechnicalInfo {simInfo} />
            </div>
        {:else}
            <!-- Error state -->
            <EmptyState
                title="No Data Available"
                description="Unable to load information for this SIM card."
                showRetry={true}
                onRetry={onRefresh}
            />
        {/if}
    </div>
{:else}
    <EmptyState
        title="No SIM Cards Found"
        description="No SIM cards are currently available."
    />
{/if}
