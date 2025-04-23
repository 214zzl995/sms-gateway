<script>
  import { fade } from "svelte/transition";
  import { apiClient } from "../js/api";

  let { selectedDevice } = $props();

  let page = $state(1);
  let messages = $state([]);
  let isLoading = $state(false);
  let error = $state(null);
  let totalPages = $state(0);
  const per_page = 10;

  $effect(() => {
    if (selectedDevice || page) {
      loadMessages();
    }
  });

  async function loadMessages() {
    isLoading = true;
    error = null;
    try {
      const response = await apiClient.getSmsPaginated(
        page,
        per_page,
        selectedDevice?.name,
      );

      if (response.status === 200) {
        messages = response.data.data || [];
        totalPages = Math.ceil(response.data.total / 10);
      }
    } catch (err) {
      error = err.message?.includes("Failed to fetch")
        ? "Network error: Unable to connect to server"
        : `Error loading messages: ${err.message}`;
    } finally {
      isLoading = false;
    }
  }

  function nextPage() {
    if (page < totalPages) page++;
  }

  function prevPage() {
    if (page > 1) page--;
  }
</script>

<div class="flex flex-col h-full max-h-[calc(100vh-150px)] overflow-hidden">
  <!-- 分页放在顶部标题栏下面 -->
  <div
    class="flex items-center justify-center gap-4 p-4 bg-gray-100 dark:bg-zinc-800 rounded-md mb-4"
  >
    <button
      onclick={prevPage}
      disabled={page === 1}
      class="px-3 py-1 border border-gray-300 rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-600 hover:text-white disabled:hover:bg-transparent"
    >
      ← Previous
    </button>
    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
      Page {page} of {totalPages}
    </span>
    <button
      onclick={nextPage}
      disabled={page === totalPages}
      class="px-3 py-1 border border-gray-300 rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-600 hover:text-white disabled:hover:bg-transparent"
    >
      Next →
    </button>
  </div>

  <!-- 消息内容滚动区 -->
  <div class="flex-1 overflow-y-auto px-4">
    {#if isLoading}
      <div
        class="flex items-center justify-center gap-2 p-8 text-gray-600 dark:text-gray-400"
      >
        <div
          class="w-6 h-6 border-4 border-gray-300 border-t-gray-600 rounded-full animate-spin"
        ></div>
        <span>Loading messages...</span>
      </div>
    {:else if error}
      <div
        class="p-8 rounded bg-red-100 text-red-700 flex flex-col items-center gap-4"
      >
        <span>⚠️ {error}</span>
        <button
          onclick={loadMessages}
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    {:else if messages.length === 0}
      <div
        class="p-8 rounded bg-gray-100 dark:bg-zinc-700 text-gray-500 dark:text-gray-400 text-center select-none"
      >
        📭 No messages found for this device
      </div>
    {:else}
      {#each messages as message (message.id)}
        <div
          transition:fade
          class="mb-6 p-4 rounded-lg shadow-md bg-white dark:bg-zinc-900 cursor-pointer
            transition-transform duration-200
            {message.sender
            ? 'border-l-4 border-blue-500'
            : 'border-l-4 border-green-500'}
            hover:translate-x-1"
        >
          <div
            class="text-xs text-gray-500 dark:text-gray-400 mb-2 select-none flex gap-2"
          >
            {#if message.sender}
              <span>sender: {message.sender}</span>
            {/if}
            {#if message.receiver}
              <span>receiver: {message.receiver}</span>
            {/if}
          </div>

          <div class="text-sm text-gray-800 dark:text-gray-200 mb-2">
            {message.message}
          </div>

          <div
            class="flex justify-between text-xs text-gray-500 dark:text-gray-400 italic select-none"
          >
            <span class="hidden md:inline-block">
              {#if !selectedDevice}
                {message.device}
              {/if}
            </span>
            <span>{message.timestamp}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
