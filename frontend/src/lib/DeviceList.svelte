<script>
    import { slide, fade } from "svelte/transition";
    import ModemIcon from "./ModemIcon.svelte";
    import ArrowIcon from "./ArrowIcon.svelte";

    let isExpanded = $state(true);

    let { selectDevice, devices, selectedDevice } = $props();

    const selectAll = () => {
        selectDevice(null);
    };

    const toggleExpand = () => {
        isExpanded = !isExpanded;
    };
</script>

<div class="w-full">
    <div
        class="font-semibold cursor-pointer rounded-md flex justify-between items-center mb-2 h-8
         focus:outline-none focus:ring-0 hover:bg-gray-100 dark:hover:bg-zinc-700 pl-2 select-none
         text-gray-900 dark:text-gray-100"
        role="button"
        tabindex="0"
        onclick={selectAll}
        onkeydown={(e) => e.key === "Enter" && selectAll()}
    >
        All Device
        <ArrowIcon {isExpanded} onclick={() => toggleExpand()} />
    </div>

    {#if isExpanded}
        <div transition:slide={{ duration: 300 }}>
            {#each $devices as device (device.name)}
                <div class="pb-2">
                    <div
                        class="px-2 py-2 rounded-md cursor-pointer transition-transform duration-200 flex items-center justify-between
                    text-[0.9rem] focus:outline-none focus:ring-0
                    dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-zinc-700 hover:text-gray-800 dark:hover:text-gray-100
                    border-b border-gray-200 dark:border-zinc-700"
                        class:bg-gray-200={selectedDevice?.name === device.name}
                        class:dark:bg-zinc-900={selectedDevice?.name ===
                            device.name}
                        class:text-gray-500={selectedDevice?.name !==
                            device.name}
                        class:dark:text-gray-400={selectedDevice?.name !==
                            device.name}
                        role="button"
                        tabindex="0"
                        onclick={() => selectDevice(device)}
                        onkeydown={(e) =>
                            e.key === "Enter" && selectDevice(device)}
                        in:fade={{ duration: 200 }}
                        out:fade
                    >
                        <div
                            class="flex flex-col items-start justify-center gap-[0.2rem]"
                        >
                            <div
                                class="flex items-center gap-2 text-base select-none"
                            >
                                {device.name}
                            </div>

                            {#if selectedDevice?.name === device.name}
                                <div
                                    class="mt-[0.2rem] text-gray-500 text-xs select-none flex gap-2 items-center dark:text-gray-400"
                                    transition:slide={{ duration: 300 }}
                                >
                                    <ModemIcon />
                                    <div>
                                        <p>Port: {device.com_port}</p>
                                        <p>Rate: {device.baud_rate}</p>
                                    </div>
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>
