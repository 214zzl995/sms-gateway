<script>
  import DeviceList from "./lib/DeviceList.svelte";
  import MessageList from "./lib/MessageList.svelte";
  import ModemIcon from "./lib/ModemIcon.svelte";
  import { devices, messages } from "./stores/devices";

  let selectedDevice = null;

  const selectDevice = (device) => (selectedDevice = device);
  const showAllMessages = () => (selectedDevice = null);
</script>

<div class="container">
  <!-- 左侧导航 -->
  <div class="nav">
    <div class="nav-top">
      <div class="icon" aria-label="设备图标"></div>
      <span>SMS</span>
    </div>

    <button on:click={() => alert("发送指令逻辑")}>Send SMS</button>

    <DeviceList {devices} {selectedDevice} {selectDevice} />
  </div>

  <!-- 主内容区域 -->
  <div class="main">
    <div class="main-header">
      {selectedDevice ? `${selectedDevice.name}` : "All"}
      {#if selectedDevice}
        <div class="modem-info">
          <div>信号强度:</div>
        </div>
      {/if}
    </div>

    <MessageList {messages} />
  </div>
</div>

<style global>
  .icon {
    width: 32px;
    height: 32px;
    background: #495057;
    border-radius: 6px;
  }
  :global(:root) {
    --primary-color: #0d6efd;
    --bg-color: #f8f9fa;
    --border-color: #dee2e6;
  }

  .container {
    display: flex;
    height: 100vh;
    width: 100vw;
    font-family: Arial, sans-serif;
  }

  .nav {
    width: 280px;
    background: var(--bg-color);
    padding: 20px;
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .nav-top {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border-color);
  }

  button {
    padding: 12px;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  button:hover {
    background: #0b5ed7;
  }

  .main {
    flex: 1;
    padding: 24px;
    background: white;
  }

  .main-header {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 24px;
    color: #212529;
    display: flex;
    justify-content: space-between;
  }
</style>
