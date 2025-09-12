<!-- frontend/src/lib/components/common/EditableField.svelte -->
<script>
    import Icon from "@iconify/svelte";
    
    let {
        value = "",
        icon = "mage:tag",
        label = "Field",
        placeholder = "Not set",
        onSave = async (val) => true,
        inputClass = "w-32"
    } = $props();
    
    let isEditing = $state(false);
    let tempValue = $state("");
    let inputRef = $state();
    
    function startEdit() {
        isEditing = true;
        tempValue = value || "";
        // Focus input after DOM update
        setTimeout(() => {
            inputRef?.focus();
        }, 0);
    }
    
    function cancelEdit() {
        isEditing = false;
        tempValue = "";
    }
    
    async function save() {
        const success = await onSave(tempValue);
        if (success) {
            cancelEdit();
        }
    }
    
    async function handleKeydown(e) {
        if (e.key === 'Enter') await save();
        if (e.key === 'Escape') cancelEdit();
    }
</script>

<div class="flex items-center justify-between">
    <div class="flex items-center">
        <Icon {icon} class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
        <div>
            <div class="text-xs text-gray-500 dark:text-gray-400">{label}</div>
            <div class="text-sm font-medium dark:text-gray-300">
                {#if isEditing}
                    <input
                        type="text"
                        bind:value={tempValue}
                        bind:this={inputRef}
                        class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-sm {inputClass}"
                        onkeydown={handleKeydown}
                    />
                {:else}
                    {value || placeholder}
                {/if}
            </div>
        </div>
    </div>
    <div class="flex items-center space-x-1">
        {#if isEditing}
            <button
                class="text-green-600 hover:text-green-800 dark:text-green-400"
                onclick={save}
            >
                <Icon icon="mage:check" width="16" height="16" />
            </button>
            <button
                class="text-gray-600 hover:text-gray-800 dark:text-gray-400"
                onclick={cancelEdit}
            >
                <Icon icon="mage:multiply" width="16" height="16" />
            </button>
        {:else}
            <button
                class="text-blue-600 hover:text-blue-800 dark:text-blue-400"
                onclick={startEdit}
            >
                <Icon icon="mage:edit" width="16" height="16" />
            </button>
        {/if}
    </div>
</div>