<script>
    import { slide, fade } from "svelte/transition";
    import ModemIcon from "./ModemIcon.svelte";
    import ArrowIcon from "./ArrowIcon.svelte";

    let isExpanded = $state(true);

    let { selectDevice, devices, selectedDevice } = $props();

    const selectAll = () => {
        selectDevice(null);
    };

    const toggleExpand = () =>{
        isExpanded = !isExpanded;
    }
</script>

<div class="list">
    <div
        class="list-header"
        role="button"
        tabindex="0"
        onclick={selectAll}
        onkeydown={(e) => e.key === "Enter" && selectAll()}
    >
        All Device
        <ArrowIcon {isExpanded} onclick={()=> toggleExpand()} />
    </div>

    {#if isExpanded}
        <div transition:slide={{ duration: 300 }}>
            {#each $devices as device (device.name)}
                <div class="list-item">
                    <div
                        class=" {selectedDevice?.name === device.name
                            ? 'selected'
                            : ''}"
                        role="button"
                        tabindex="0"
                        onclick={() => selectDevice(device)}
                        onkeydown={(e) =>
                            e.key === "Enter" && selectDevice(device)}
                        in:fade={{ duration: 200 }}
                        out:fade
                    >
                        <div class="device-info">
                            <div class="device-name">
                                <ModemIcon />
                                {device.name}
                            </div>
                            <div class="device-connect">
                                {device.com_port}
                                {device.baud_rate}
                            </div>
                            
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .list-header {
        padding: 12px;
        font-weight: 600;
        cursor: pointer;
        background: #e9ecef;
        border-radius: 6px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }

    .list-item {
        padding-bottom: 8px;
    }

    .list-item > div {
        padding: 12px;
        background: white;
        border-radius: 6px;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
        cursor: pointer;
        transition: transform 0.2s;
        height: 3rem;
        font-size: 0.85rem;
    }

    .list-item:hover {
        /* transform: translateX(4px); */
        background: #f0f7ff;
    }

    .list-item > div.selected {
        background: #e7f1ff;
        /* border-left: 3px solid var(--primary-color); */
    }

    .device-info {
        display: flex;
        align-items: start;
        justify-content: center;
        flex-direction: column;
        height: 100%;
        gap:0.2rem;
    
    }

    .device-name {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 1rem;
    }

    .device-connect{
        margin-top: 0.2rem;
        color: rgb(121, 121, 121);
    }
</style>
