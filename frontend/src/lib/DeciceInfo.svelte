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

<div class="device-info-card">
    {#if isModemInfoLoading}
        <div class="status-message loading">
            <div class="spinner"></div>
            正在加载设备信息...
        </div>
    {:else if modemInfoError}
        <div class="status-message error">
            ⚠️ {modemInfoError}
            <button onclick={loadModemInfo}>重试</button>
        </div>
    {:else if modemInfo}
        <div class="info-grid">
            <div class="info-item">
                <span class="label">信号强度:</span>
                <span class="value">{modemInfo.signal_quality.data.rssi} dBm</span>
            </div>
            <div class="info-item">
                <span class="label">网络状态:</span>
                <span class="value">{modemInfo.network_registration.data.status}</span>
            </div>
            <div class="info-item">
                <span class="label">运营商:</span>
                <span class="value">{modemInfo.operator.data.operator_name}</span>
            </div>
            <div class="info-item">
                <span class="label">设备型号:</span>
                <span class="value">{modemInfo.modem_model.data.model}</span>
            </div>
        </div>
    {/if}
</div>

<style>
    .device-info-card {
        background: white;
        border-radius: 8px;
        padding: 0.5rem;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        font-size: 0.7rem;
    }

    .info-grid {
        display: flex;
        gap: 1rem;
    }

    .info-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem;
        background: #f8f9fa;
        border-radius: 4px;
    }

    .label {
        font-weight: 500;
        color: #666;
    }

    .value {
        color: var(--primary-color);
        font-weight: 600;
    }
</style>
