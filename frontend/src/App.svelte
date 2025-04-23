<script>
  import { onMount } from "svelte";
  import { updateStorageValue } from "./js/storage";
  import DeciceInfo from "./lib/DeciceInfo.svelte";
  import DeviceList from "./lib/DeviceList.svelte";
  import LogoutIcon from "./lib/LogoutIcon.svelte";
  import MessageList from "./lib/MessageList.svelte";
  import SendDialog from "./lib/SendDialog.svelte";
  import Edit from "./lib/Edit.svelte";
  import { devices, initDeices } from "./stores/devices";

  let sendDialogShow = $state(false);

  initDeices();

  let selectedDevice = $state(null);

  const selectDevice = (
    /** @type {{ name: string | number | boolean; }} */ device,
  ) => {
    selectedDevice = device;
    if (device) {
      history.replaceState(null, null, `#${encodeURIComponent(device.name)}`);
    } else {
      history.replaceState(null, null, " ");
    }
  };

  const logout = async () => {
    await updateStorageValue("auth", null);
    window.location.reload();
  };

  onMount(() => {
    const hashValue = window.location.hash.substring(1);
    console.log(hashValue);
    if (hashValue) {
      const decodedHash = decodeURIComponent(hashValue);
      const unsubscribe = devices.subscribe(($devices) => {
        const target = $devices.find((d) => d.name === decodedHash);
        if (target) {
          selectedDevice = target;
        }
      });
      return unsubscribe;
    }
  });
</script>

<div class="flex h-screen w-screen font-sans dark:bg-zinc-900 dark:text-white">
  <!-- 左侧导航 -->
  <div
    class="w-1/6 bg-zinc-100 dark:bg-zinc-800 p-2 border-r border-gray-100 dark:border-zinc-700 flex flex-col gap-4"
  >
    <button
      class="ml-2 mr-2 border-2 border-dashed hover:border-solid rounded-md px-6 py-2 flex gap-5 dark:border-zinc-500 dark:hover:border-zinc-300"
      onclick={() => (sendDialogShow = true)}
    >
      <Edit />
      <div class="w-full text-sm flex justify-end items-center font-semibold">
        Send SMS
      </div>
    </button>

    <DeviceList {devices} {selectedDevice} {selectDevice} />

    <div class="flex flex-1 items-end mb-2">
      <button
        class="w-full flex justify-start items-center gap-2 bg-zinc-100 hover:bg-zinc-200 p-4 rounded-md dark:bg-zinc-900 dark:hover:bg-zinc-600"
        onclick={logout}
      >
        <LogoutIcon /> Logout
      </button>
    </div>
  </div>

  <!-- 主内容区域 -->
  <div class="flex-1 p-6 bg-white dark:bg-zinc-900">
    <div
      class="text-[20px] font-semibold mb-6 text-[#212529] dark:text-zinc-200 flex justify-between"
    >
      {selectedDevice ? `${selectedDevice.name}` : "All"}
      {#if selectedDevice}
        <DeciceInfo {selectedDevice} />
      {/if}
    </div>
    <MessageList {selectedDevice} />
  </div>
</div>

<SendDialog bind:value={sendDialogShow} />
