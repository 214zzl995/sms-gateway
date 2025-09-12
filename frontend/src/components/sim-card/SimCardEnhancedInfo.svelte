<!-- frontend/src/lib/components/simcard/SimCardEnhancedInfo.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let { simInfo = null } = $props();
    
    const hasEnhancedInfo = $derived(
        simInfo && (
            simInfo.sms_center || 
            simInfo.sim_status || 
            simInfo.memory_status || 
            simInfo.com_port || 
            simInfo.signal_quality || 
            simInfo.operator_info?.operator_id
        )
    );
</script>

{#if hasEnhancedInfo}
    <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
        <h5 class="text-sm font-medium text-gray-600 dark:text-gray-400">Enhanced Information</h5>
        
        {#if simInfo.sms_center}
            <div class="flex items-center text-sm">
                <Icon icon="mage:message-dots" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">SMS Center:</span>
                <span class="font-medium dark:text-gray-300 ml-2">{simInfo.sms_center}</span>
            </div>
        {/if}
        
        {#if simInfo.sim_status}
            <div class="flex items-center text-sm">
                <Icon icon="mage:shield-check" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">SIM Status:</span>
                <span class="font-medium dark:text-gray-300 ml-2">{simInfo.sim_status}</span>
            </div>
        {/if}
        
        {#if simInfo.memory_status}
            <div class="flex items-center text-sm">
                <Icon icon="mage:memory-card" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">Memory:</span>
                <span class="font-medium dark:text-gray-300 ml-2">{simInfo.memory_status}</span>
            </div>
        {/if}
        
        {#if simInfo.com_port}
            <div class="flex items-center text-sm">
                <Icon icon="mage:link" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">Port:</span>
                <span class="font-medium dark:text-gray-300 ml-2">{simInfo.com_port} @ {simInfo.baud_rate}</span>
            </div>
        {/if}
        
        {#if simInfo.signal_quality}
            <div class="flex items-center text-sm">
                <Icon icon="mage:chart-up-b" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">Signal Detail:</span>
                <span class="font-medium dark:text-gray-300 ml-2">RSSI: {simInfo.signal_quality.rssi}, BER: {simInfo.signal_quality.ber}</span>
            </div>
        {/if}
        
        {#if simInfo.operator_info?.operator_id}
            <div class="flex items-center text-sm">
                <Icon icon="mage:building-b" class="w-4 h-4 mr-2 text-gray-500" />
                <span class="text-gray-500 dark:text-gray-400">Operator ID:</span>
                <span class="font-medium dark:text-gray-300 ml-2">{simInfo.operator_info.operator_id}</span>
            </div>
        {/if}
    </div>
{/if}