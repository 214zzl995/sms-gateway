<script>
  import { updateStorageValue } from "../js/storage";
  import Sidebar from "../components/layout/Sidebar.svelte";
  import MessageList from "../components/message/MessageList.svelte";
  import SimCardModal from "../components/sim-card/SimCardModal.svelte";

  let modemInfoCardOpen = $state(false);

  const logout = async () => {
    await updateStorageValue("auth", null);
    window.location.reload();
  };

  function handleSimCardClick() {
    modemInfoCardOpen = true;
  }

  function handleModalClose() {
    modemInfoCardOpen = false;
  }
</script>

<div class="flex h-screen w-screen font-sans dark:bg-zinc-900 dark:text-white">
  <Sidebar 
    onSimCardClick={handleSimCardClick}
    onLogoutClick={logout}
  />
  
  <!-- 主内容区域 -->
  <div class="flex-1 bg-white dark:bg-zinc-900">
    <MessageList />
  </div>
</div>

<SimCardModal
  isOpen={modemInfoCardOpen}
  onClose={handleModalClose}
/>