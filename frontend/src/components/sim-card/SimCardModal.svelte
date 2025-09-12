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
        class="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-700"
    >
        <h2 class="text-xl font-bold text-gray-800 dark:text-gray-200">
            SIM Card Information
        </h2>
        <button
            class="text-stone-800 text-xl p-2 rounded-full bg-gray-200 dark:bg-gray-700
                   transition-colors duration-500 hover:bg-gray-300 dark:hover:bg-gray-600
                   hover:text-gray-800 dark:hover:text-gray-100 dark:text-gray-100"
            onclick={() => onClose()}
        >
            <Icon icon="mage:multiply-square" width="20" height="20" />
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
