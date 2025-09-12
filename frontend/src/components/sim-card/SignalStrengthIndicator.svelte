<!-- frontend/src/lib/components/simcard/SignalStrengthIndicator.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let { rssi = 99 } = $props();
    
    function getSignalBars(rssi) {
        if (rssi === 99 || rssi < 2) return 0;
        if (rssi >= 20) return 5;
        if (rssi >= 15) return 4;
        if (rssi >= 10) return 3;
        if (rssi >= 5) return 2;
        return 1;
    }
    
    function getSignalStrengthLabel(bars) {
        const signalLabels = [
            "No Signal",
            "Very Poor", 
            "Poor",
            "Moderate",
            "Good",
            "Excellent",
        ];
        return signalLabels[bars];
    }
    
    const bars = $derived(getSignalBars(rssi));
    const label = $derived(getSignalStrengthLabel(bars));
</script>

<div class="flex items-center">
    <Icon icon="mage:chart-up-b" class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
    <div class="flex-grow">
        <div class="text-xs text-gray-500 dark:text-gray-400">Signal Strength</div>
        <div class="flex items-center">
            <div class="text-sm font-medium mr-2 dark:text-gray-300">
                {label}
            </div>
            <div class="flex space-x-1">
                {#each Array(5) as _, i}
                    <div
                        class="w-1 h-2 rounded-sm {i < bars ? 'bg-green-500 dark:bg-green-400' : 'bg-gray-300 dark:bg-gray-600'}"
                    ></div>
                {/each}
            </div>
        </div>
    </div>
</div>