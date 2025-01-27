<script>
  import { updateStorageValue } from "./js/storage";
  import DeviceList from "./lib/DeviceList.svelte";
  import LogoutIcon from "./lib/LogoutIcon.svelte";
  import MessageList from "./lib/MessageList.svelte";
  import SendDialog from "./lib/SendDialog.svelte";
  import { devices, initDeices } from "./stores/devices";

  let sendDialogShow = $state(false);

  initDeices();

  let selectedDevice = $state(null);

  const selectDevice = (device) => (selectedDevice = device);

  const logout = async () => {
    await updateStorageValue("auth", null);
    window.location.reload();
  };
</script>

<div class="container">
  <!-- 左侧导航 -->
  <div class="nav">
    <div class="nav-top">
      <div class="icon" aria-label="设备图标"></div>
      <span>SMS</span>
    </div>

    <button onclick={() => (sendDialogShow = true)}>Send SMS</button>

    <DeviceList {devices} {selectedDevice} {selectDevice} />

    <div class="logout">
      <button onclick={logout}> <LogoutIcon /> Logout</button>
    </div>
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

    <MessageList {selectedDevice} />
  </div>
</div>
<SendDialog bind:value={sendDialogShow} />

<style global>
  .icon {
    width: 32px;
    height: 32px;
    background: #495057;
    border-radius: 6px;
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

  .logout {
    flex: 1;
    display: flex;
    align-items: end;
  }

  .logout > button {
    width: 100%;
    display: flex;
    justify-content: start;
    align-items: center;
    gap: 10px;
    background: #f8f9fa;
    color: #212529;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  .logout > button:hover {
    /* transform: translateX(4px); */
    background: #f0f7ff;
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
