<script>
  import { onMount } from "svelte";
  import { updateStorageValue } from "./js/storage";
  import ConversationList from "./lib/ConversationList.svelte";
  import SendDialog from "./lib/SendDialog.svelte";
  import Icon from "@iconify/svelte";
  import ModemInfoCard from "./lib/ModemInfoCard.svelte";
  import MessageList from "./lib/MessageList.svelte";

  let modemInfoCardOpen = $state(false);

  const logout = async () => {
    await updateStorageValue("auth", null);
    window.location.reload();
  };

  onMount(() => {
    const hashValue = window.location.hash.substring(1);
    
  });
</script>

<div class="flex h-screen w-screen font-sans dark:bg-zinc-900 dark:text-white">
  <div
    class="w-1/5 bg-zinc-100 dark:bg-zinc-800 p-2 border-r border-gray-100 dark:border-zinc-700 flex flex-col h-screen"
  >
    <div class="flex-1 overflow-hidden bg-zinc-100 dark:bg-zinc-800">
      <ConversationList />
    </div>

    <div class="flex flex-col gap-1 mt-2 bg-zinc-100 dark:bg-zinc-800 relative z-10">
      <button
        class="w-full flex justify-start items-center gap-2 bg-zinc-100 hover:bg-zinc-200 rounded-md dark:bg-zinc-900 dark:hover:bg-zinc-600 p-2"
        onclick={() => {
          modemInfoCardOpen = true;
        }}
      >
        <Icon icon="mage:memory-card-fill" class="text-gray-400 w-4 h-4" />
        <span>Devices</span>
      </button>
      <button
        class="w-full flex justify-start items-center gap-2 bg-zinc-100 hover:bg-zinc-200 rounded-md dark:bg-zinc-900 dark:hover:bg-zinc-600 p-2"
        onclick={logout}
      >
        <Icon icon="mage:logout" class="text-gray-400 w-4 h-4" />
        <span>Logout</span>
      </button>
    </div>
  </div>

  <!-- 主内容区域 -->
  <div class="flex-1 bg-white dark:bg-zinc-900">
    <MessageList />

  </div>
</div>

<ModemInfoCard isOpen={modemInfoCardOpen} onClose={() => (modemInfoCardOpen = false)} />

