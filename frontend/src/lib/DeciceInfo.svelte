<script>
    import { apiClient } from "../js/api";

    let { selectedDevice } = $props();

    let isModemInfoLoading = $state(false);
    let modemInfoError = $state(null);
    let modemInfo = $state(null);

    $effect(() => {
        if (selectedDevice) {
            loadModemInfo();
        }
    });

    async function loadModemInfo() {
        isModemInfoLoading = true;
        modemInfoError = null;
        try {
            const response = await apiClient.getModemsInfo(selectedDevice.name);
            modemInfo = response.data;
        } catch (err) {
            modemInfoError = err.message?.includes("Failed to fetch")
                ? "无法获取设备信息"
                : `错误: ${err.message}`;
        } finally {
            isModemInfoLoading = false;
        }
    }
</script>

<div class="bg-white dark:bg-zinc-900 rounded-md p-2 shadow-sm text-xs">
    {#if isModemInfoLoading}
      <div class="flex items-center gap-2 p-4 text-gray-600 dark:text-gray-400">
        <div class="w-5 h-5 border-4 border-gray-300 border-t-gray-600 rounded-full animate-spin"></div>
        正在加载设备信息...
      </div>
    {:else if modemInfoError}
      <div class="p-4 rounded bg-red-100 dark:bg-red-900 text-red-600 dark:text-red-400 flex flex-col items-center gap-3">
        <span>⚠️ {modemInfoError}</span>
        <button 
          onclick={loadModemInfo} 
          class="px-4 py-1 rounded bg-red-600 text-white hover:bg-red-700"
        >
          重试
        </button>
      </div>
    {:else if modemInfo}
      <div class="flex gap-4">
        <div class="flex justify-between items-center p-2 bg-gray-100 dark:bg-zinc-800 rounded-md flex-1">
          <span class="font-semibold text-gray-600 dark:text-gray-400">信号强度:</span>
          <span class="font-semibold text-primary-600 dark:text-primary-400">{modemInfo.signal_quality.data.rssi} dBm</span>
        </div>
        <div class="flex justify-between items-center p-2 bg-gray-100 dark:bg-zinc-800 rounded-md flex-1">
          <span class="font-semibold text-gray-600 dark:text-gray-400">网络状态:</span>
          <span class="font-semibold text-primary-600 dark:text-primary-400">{modemInfo.network_registration.data.status}</span>
        </div>
        <div class="flex justify-between items-center p-2 bg-gray-100 dark:bg-zinc-800 rounded-md flex-1">
          <span class="font-semibold text-gray-600 dark:text-gray-400">运营商:</span>
          <span class="font-semibold text-primary-600 dark:text-primary-400">{modemInfo.operator.data.operator_name}</span>
        </div>
        <div class="flex justify-between items-center p-2 bg-gray-100 dark:bg-zinc-800 rounded-md flex-1">
          <span class="font-semibold text-gray-600 dark:text-gray-400">设备型号:</span>
          <span class="font-semibold text-primary-600 dark:text-primary-400">{modemInfo.modem_model.data.model}</span>
        </div>
      </div>
    {/if}
  </div>
