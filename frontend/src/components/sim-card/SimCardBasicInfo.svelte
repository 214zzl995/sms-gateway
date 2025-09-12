<!-- frontend/src/lib/components/simcard/SimCardBasicInfo.svelte -->
<script>
    import Icon from "@iconify/svelte";
    import EditableField from "../common/EditableField.svelte";
    
    let {
        simCard = {},
        simInfo = null,
        onUpdatePhone = async (phone) => true,
        onUpdateAlias = async (alias) => true
    } = $props();
    
    function getStatusDescription(status) {
        const statusMap = {
            "0": "Not registered",
            "1": "Registered (Home)",
            "2": "Searching",
            "3": "Registration denied", 
            "5": "Registered (Roaming)"
        };
        return statusMap[status] || `Status ${status}`;
    }
</script>

<div class="space-y-4">
    <h4 class="text-md font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-600 pb-2">
        Basic Information
    </h4>
    
    <EditableField
        value={simCard.phone_number}
        icon="mage:phone"
        label="Phone Number"
        placeholder="Not set"
        onSave={onUpdatePhone}
    />
    
    <EditableField
        value={simCard.alias}
        icon="mage:tag"
        label="Alias"
        placeholder="Not set"
        onSave={onUpdateAlias}
    />
    
    <div class="flex items-center">
        <Icon icon="mage:globe" class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
        <div>
            <div class="text-xs text-gray-500 dark:text-gray-400">Network Status</div>
            <div class="text-sm font-medium dark:text-gray-300">
                {simInfo?.operator_info?.registration_status ? getStatusDescription(simInfo.operator_info.registration_status) : 'Unknown'}
            </div>
        </div>
    </div>
    
    <div class="flex items-center">
        <Icon icon="mage:building-b" class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
        <div>
            <div class="text-xs text-gray-500 dark:text-gray-400">Operator</div>
            <div class="text-sm font-medium dark:text-gray-300">
                {simInfo?.operator_info?.operator_name || 'Unknown'}
            </div>
        </div>
    </div>
</div>