<!-- frontend/src/lib/components/simcard/SimCardModal.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import { onMount } from "svelte";
    import { simCards, simCardActions } from "../../stores/simcards";
    import { apiClient } from "../../js/api.js";
    import Modal from "../common/Modal.svelte";
    import SimCardTabs from "./SimCardTabs.svelte";
    import SimCardTabContent from "./SimCardTabContent.svelte";

    // Props
    let { isOpen = false, onClose = () => {} } = $props();

    // State management
    let activeSimId = $state(null);
    let tabDataCache = $state({});
    let isCurrentTabLoading = $state(false);

    // Load SIM cards on mount
    onMount(async () => {
        await simCardActions.loadAll();
    });

    // Track modal open state and initialize
    let previousIsOpen = false;
    $effect(() => {
        if (isOpen && !previousIsOpen && $simCards.length > 0) {
            if (!activeSimId || !$simCards.find((s) => s.id === activeSimId)) {
                activeSimId = $simCards[0]?.id;
            }
            if (activeSimId) {
                loadTabData(activeSimId);
            }
            previousIsOpen = true;
        } else if (!isOpen) {
            previousIsOpen = false;
        }
    });

    // Load data for specific SIM tab
    /**
     * @param {string | number} simId
     */
    async function loadTabData(simId) {
        if (!simId) return;

        isCurrentTabLoading = true;

        try {
            const response = await apiClient.getSimInfo(simId);
            const simInfo = response.data || response;
            tabDataCache[simId] = simInfo;
            tabDataCache = { ...tabDataCache };
        } catch (error) {
            console.error(`Failed to load SIM info for ${simId}:`, error);
            tabDataCache[simId] = null;
            tabDataCache = { ...tabDataCache };
        } finally {
            isCurrentTabLoading = false;
        }
    }

    // Switch to different tab
    function switchTab(simId) {
        if (activeSimId === simId) return;

        activeSimId = simId;
        if (!tabDataCache[simId]) {
            loadTabData(simId);
        }
    }

    // Refresh current tab data
    function refreshCurrentTab() {
        if (activeSimId) {
            loadTabData(activeSimId);
        }
    }

    // Update handlers
    async function handleUpdatePhone(simId, phone) {
        const success = await simCardActions.updatePhoneNumber(simId, phone);
        if (success) {
            // Refresh cache if needed
            if (tabDataCache[simId]) {
                await loadTabData(simId);
            }
        }
        return success;
    }

    async function handleUpdateAlias(simId, alias) {
        const success = await simCardActions.updateAlias(simId, alias);
        if (success) {
            // Refresh cache if needed
            if (tabDataCache[simId]) {
                await loadTabData(simId);
            }
        }
        return success;
    }

    // Helper functions
    function getSimDisplayName(simCard) {
        return (
            simCard.alias ||
            simCard.phone_number ||
            `SIM ${simCard.id.slice(-8)}`
        );
    }

    // Derived state
    const activeSimCard = $derived(
        $simCards.find((sim) => sim.id === activeSimId),
    );
    const activeSimInfo = $derived(tabDataCache[activeSimId]);
</script>

<Modal {isOpen} {onClose}>
    <!-- Modal Header -->
    <div
        class="flex justify-between items-center p-6 border-b border-gray-200 dark:border-zinc-700"
    >
        <div class="flex items-center gap-3">
            <div class="w-8 h-8 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center">
                <Icon icon="carbon:sim-card" class="w-4 h-4 text-gray-100 dark:text-gray-900" />
            </div>
            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100">
                SIM Card Information
            </h2>
        </div>
        <button
            class="p-2 rounded-lg bg-gray-100 dark:bg-zinc-800 text-gray-600 dark:text-gray-400
                   transition-colors duration-200 hover:bg-gray-200 dark:hover:bg-zinc-700
                   hover:text-gray-800 dark:hover:text-gray-200"
            onclick={() => onClose()}
        >
            <Icon icon="carbon:close" class="w-5 h-5" />
        </button>
    </div>

    <!-- Tab Headers -->
    <SimCardTabs
        simCards={$simCards}
        {activeSimId}
        onTabSwitch={switchTab}
        getDisplayName={getSimDisplayName}
    />

    <!-- Tab Content -->
    <div class="flex-1 overflow-auto p-4">
        <SimCardTabContent
            simCard={activeSimCard}
            simInfo={activeSimInfo}
            isLoading={isCurrentTabLoading}
            onRefresh={refreshCurrentTab}
            onUpdatePhone={(phone) => handleUpdatePhone(activeSimId, phone)}
            onUpdateAlias={(alias) => handleUpdateAlias(activeSimId, alias)}
            getDisplayName={getSimDisplayName}
        />
    </div>
</Modal>
