<script>
    import Icon from "@iconify/svelte";
    import { fade } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import { devices } from "../stores/devices";

    // Props to control visibility from parent component
    export let isOpen = false;
    export let onClose = () => (isOpen = false);


    // Function to get status description based on status code
    function getStatusDescription(status) {
        const statusMap = {
            "0": "Not registered",
            "1": "Registered, home network",
            "2": "Searching",
            "3": "Registration denied",
            "4": "Unknown",
            "5": "Registered, roaming",
        };
        return statusMap[status] || "Unknown";
    }

    // Function to translate RSSI value to signal bars (0-5)
    function getSignalBars(rssi) {
        if (rssi === 99 || rssi === 31) return 0; // No signal or unknown
        if (rssi >= 0 && rssi <= 31) {
            return Math.ceil((31 - rssi) / 6); // Convert to 0-5 scale
        }
        return 0;
    }

    // Function to get a human-readable signal strength label
    function getSignalStrengthLabel(bars) {
        const signalLabels = [
            "No Signal",
            "Very Weak",
            "Weak",
            "Moderate",
            "Good",
            "Excellent",
        ];
        return signalLabels[bars];
    }
</script>

{#if isOpen}
    <div
        class="fixed inset-0 flex items-center justify-center
         bg-zinc-100 dark:bg-zinc-800 bg-opacity-50 dark:bg-opacity-30 z-50 backdrop-blur-md"
        transition:fade={{ duration: 200 }}
    >
        <div
            class="bg-zinc-100 dark:bg-zinc-800 rounded-lg shadow-xl w-2/3 max-w-2xl p-4"
            transition:fade={{ delay: 100, duration: 300, easing: cubicOut }}
        >
            <div class="flex justify-between items-center mb-4">
                <h2 class="text-xl font-bold text-gray-800 dark:text-gray-200">
                    Modem Information
                </h2>

                <button
                    class="text-stone-800 text-xl p-2 rounded-full bg-gray-200 dark:bg-gray-700 ml-2
                           transition-colors duration-500 hover:bg-gray-300 dark:hover:bg-gray-600
                           hover:text-gray-800 dark:hover:text-gray-100 dark:text-gray-100"
                    on:click={onClose}
                >
                    <Icon icon="mage:multiply-square" width="20" height="20" />
                </button>
            </div>

            <div
                class="grid grid-cols-1 md:grid-cols-2 gap-3 overflow-y-auto max-h-[80vh]"
            >
                {#each $devices as modem}
                    <div
                        class="rounded-lg p-3 border border-gray-200 dark:border-gray-700 hover:shadow-md transition-shadow"
                    >
                        <div class="flex justify-between items-center mb-2">
                            <div class="flex items-center">
                                <h3
                                    class="text-base font-semibold text-gray-700 dark:text-gray-300"
                                >
                                    {modem.name}
                                </h3>
                            </div>

                            <span
                                class="px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs font-medium rounded-full"
                            >
                                {modem.modem_model.data.model}
                            </span>
                        </div>

                        <div class="space-y-2">
                            <!-- Network Status -->
                            <div class="flex items-center">
                                <div
                                    class="w-5 h-5 mr-2 text-gray-500 dark:text-gray-400"
                                >
                                    <Icon
                                        icon="mage:globe"
                                        width="20"
                                        height="20"
                                    />
                                </div>
                                <div>
                                    <div
                                        class="text-xs text-gray-500 dark:text-gray-400"
                                    >
                                        Network Status
                                    </div>
                                    <div
                                        class="text-sm font-medium dark:text-gray-300"
                                    >
                                        {getStatusDescription(
                                            modem.network_registration.data
                                                .status,
                                        )}
                                    </div>
                                </div>
                            </div>

                            <!-- Operator -->
                            <div class="flex items-center">
                                <div
                                    class="w-5 h-5 mr-2 text-gray-500 dark:text-gray-400"
                                >
                                    <Icon
                                        icon="mage:building-b"
                                        width="20"
                                        height="20"
                                    />
                                </div>
                                <div>
                                    <div
                                        class="text-xs text-gray-500 dark:text-gray-400"
                                    >
                                        Operator
                                    </div>
                                    <div
                                        class="text-sm font-medium dark:text-gray-300"
                                    >
                                        {modem.operator.data.operator_name}
                                    </div>
                                </div>
                            </div>

                            <!-- Signal Strength -->
                            <div class="flex items-center">
                                <div
                                    class="w-5 h-5 mr-2 text-gray-500 dark:text-gray-400"
                                >
                                    <Icon
                                        icon="mage:chart-up-b"
                                        width="20"
                                        height="20"
                                    />
                                </div>
                                <div class="flex-grow">
                                    <div
                                        class="text-xs text-gray-500 dark:text-gray-400"
                                    >
                                        Signal Strength
                                    </div>
                                    <div class="flex items-center">
                                        <div
                                            class="text-sm font-medium mr-2 dark:text-gray-300"
                                        >
                                            {getSignalStrengthLabel(
                                                getSignalBars(
                                                    modem.signal_quality.data
                                                        .rssi,
                                                ),
                                            )}
                                        </div>
                                        <div class="flex space-x-1">
                                            {#each Array(5) as _, i}
                                                <div
                                                    class="w-1 h-2 rounded-sm {i <
                                                    getSignalBars(
                                                        modem.signal_quality
                                                            .data.rssi,
                                                    )
                                                        ? 'bg-green-500 dark:bg-green-400'
                                                        : 'bg-gray-300 dark:bg-gray-600'}"
                                                ></div>
                                            {/each}
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <!-- Location Area -->
                            <div class="flex items-center">
                                <div
                                    class="w-5 h-5 mr-2 text-gray-500 dark:text-gray-400"
                                >
                                    <Icon
                                        icon="mage:map-marker"
                                        width="20"
                                        height="20"
                                    />
                                </div>
                                <div>
                                    <div
                                        class="text-xs text-gray-500 dark:text-gray-400"
                                    >
                                        Location Area
                                    </div>
                                    <div
                                        class="text-sm font-medium dark:text-gray-300"
                                    >
                                        LAC: {modem.network_registration.data
                                            .location_area_code || "N/A"}
                                        {#if modem.network_registration.data.cell_id}
                                            <span class="ml-2"
                                                >Cell ID: {modem
                                                    .network_registration.data
                                                    .cell_id}</span
                                            >
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}
