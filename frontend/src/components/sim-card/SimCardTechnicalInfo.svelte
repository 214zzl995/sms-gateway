<!-- frontend/src/lib/components/simcard/SimCardTechnicalInfo.svelte -->
<script>
    import SignalStrengthIndicator from "./SignalStrengthIndicator.svelte";
    import SimCardEnhancedInfo from "./SimCardEnhancedInfo.svelte";
    
    let { simInfo = null } = $props();
</script>

<div class="space-y-4">
    <h4 class="text-md font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-600 pb-2">
        Technical Details
    </h4>
    
    <SignalStrengthIndicator rssi={simInfo?.signal_quality?.rssi || 99} />
    
    <SimCardEnhancedInfo {simInfo} />
</div>