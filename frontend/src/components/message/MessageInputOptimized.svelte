<script>
  import Icon from "@iconify/svelte";
  import SimSelector from "./SimSelector.svelte";
  import Modal from "../common/Modal.svelte";

  let {
    sendMessageContent = $bindable(""),
    showNewMessage = false,
    concatInputText = "",
    onSend = () => {},
  } = $props();

  let isComposing = $state(false);
  let selectedSim = $state(null);
  let showConfirmDialog = $state(false);

  function handleSendClick() {
    if (showNewMessage && !concatInputText.trim()) {
      return;
    }

    if (!sendMessageContent.trim()) {
      return;
    }

    if (!selectedSim) {
      alert("Please select a SIM card first");
      return;
    }

    showConfirmDialog = true;
  }

  function confirmSend() {
    if (selectedSim) {
      onSend(selectedSim.id);
      showConfirmDialog = false;
      sendMessageContent = ""; // 清空输入框
    }
  }

  function cancelSend() {
    showConfirmDialog = false;
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && !isComposing) {
      handleSendClick();
    }
  }
</script>

<div
  class="absolute bottom-0 left-0 right-0 h-20 bg-gray-50/95 dark:bg-zinc-900/95 backdrop-blur-xl border-t border-gray-200 dark:border-zinc-700 z-10"
>
  <div class="flex items-center gap-3 h-full pr-6 max-w-6xl mx-auto">
    <div class="flex-1 flex items-center gap-3 relative">
      <div
        class="flex-1 transition-all duration-300 ease-out relative"
        class:opacity-50={showNewMessage && !concatInputText.trim()}
        class:pointer-events-none={showNewMessage && !concatInputText.trim()}
      >
        <div class="relative">
          <Icon
            icon="carbon:chat"
            class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500 dark:text-gray-400 pointer-events-none"
          />
          <input
            type="text"
            bind:value={sendMessageContent}
            oncompositionstart={() => (isComposing = true)}
            oncompositionend={() => (isComposing = false)}
            onkeydown={handleKeyDown}
            disabled={showNewMessage && !concatInputText.trim()}
            placeholder={showNewMessage && !concatInputText.trim()
              ? "Enter contact first"
              : "Type your message..."}
            class="w-full h-12 pl-11 pr-4 bg-white dark:bg-zinc-800 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 transition-all duration-200 outline-none focus:border-gray-500 dark:focus:border-zinc-500 hover:border-gray-400 dark:hover:border-zinc-600"
          />
        </div>
      </div>
    </div>

    <div class="h-6 border-l border-gray-300 dark:border-zinc-600"></div>

    <SimSelector bind:selectedSim />
    
    <div class="h-6 border-l border-gray-300 dark:border-zinc-600"></div>

    <button
      onclick={handleSendClick}
      disabled={(showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim()}
      class="flex items-center gap-2 px-5 h-12 rounded-lg font-medium text-sm transition-all duration-200 {(showNewMessage && !concatInputText.trim()) || !sendMessageContent.trim()
        ? 'bg-gray-200 dark:bg-zinc-700 text-gray-600 dark:text-gray-500 cursor-not-allowed opacity-50'
        : 'bg-gray-800 dark:bg-gray-100 text-gray-100 dark:text-gray-900 hover:bg-gray-700 dark:hover:bg-gray-200 active:scale-[0.98] cursor-pointer'}"
    >
      <Icon
        icon="carbon:send-filled"
        class="w-5 h-5"
      />
      <span>Send</span>
    </button>
  </div>
</div>

<Modal 
  isOpen={showConfirmDialog} 
  onClose={cancelSend}
  maxWidth="max-w-md"
>
  {#snippet children()}
    <div class="p-8">
      <!-- 标题区域 -->
      <div class="mb-8 text-center">
        <div class="inline-flex items-center justify-center w-12 h-12 bg-gray-900 dark:bg-gray-100 rounded-lg mb-4">
          <Icon icon="carbon:send-alt" class="w-6 h-6 text-gray-100 dark:text-gray-900" />
        </div>
        <h3 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
          Confirm Message
        </h3>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Review message details before sending
        </p>
      </div>

      <!-- SIM 卡信息 -->
      <div class="mb-6">
        <p class="text-xs text-gray-500 dark:text-gray-400 uppercase tracking-widest mb-3">
          Sending From
        </p>
        <div class="bg-gray-50 dark:bg-zinc-800/50 rounded-lg p-4 border border-gray-200 dark:border-zinc-700">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 bg-gray-800 dark:bg-gray-200 rounded-lg flex items-center justify-center flex-shrink-0">
              <Icon icon="carbon:sim-card" class="w-5 h-5 text-gray-200 dark:text-gray-800" />
            </div>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">
                {selectedSim ? selectedSim.alias : 'Not Selected'}
              </p>
              <p class="text-xs text-gray-500 dark:text-gray-400 font-mono mt-0.5">
                {selectedSim ? selectedSim.phone_number : '—'}
              </p>
            </div>
            <div class="flex items-center gap-1.5">
              <div class="w-1.5 h-1.5 bg-green-500 rounded-full"></div>
              <span class="text-xs text-gray-500 dark:text-gray-400">Active</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 消息内容 -->
      {#if sendMessageContent}
        <div class="mb-6">
          <div class="flex items-center justify-between mb-3">
            <p class="text-xs text-gray-500 dark:text-gray-400 uppercase tracking-widest">
              Message Content
            </p>
            <span class="text-xs font-mono text-gray-400 dark:text-gray-500">
              {sendMessageContent.length} chars
            </span>
          </div>
          <div class="bg-white dark:bg-zinc-900 rounded-lg p-4 border border-gray-200 dark:border-zinc-700 max-h-32 overflow-y-auto">
            <p class="text-sm text-gray-700 dark:text-gray-300 leading-relaxed whitespace-pre-wrap">
              {sendMessageContent}
            </p>
          </div>
        </div>
      {/if}

      <!-- 费用提醒 -->
      <div class="mb-8 p-3 bg-gray-100 dark:bg-zinc-800 rounded-lg border border-gray-200 dark:border-zinc-700">
        <div class="flex items-center gap-2">
          <Icon icon="carbon:information" class="w-4 h-4 text-gray-500 dark:text-gray-400 flex-shrink-0" />
          <p class="text-xs text-gray-600 dark:text-gray-400 leading-relaxed">
            Standard SMS rates apply • Message sends immediately
          </p>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="grid grid-cols-2 gap-3">
        <button
          onclick={cancelSend}
          class="px-5 py-3 bg-white dark:bg-zinc-800 text-gray-700 dark:text-gray-300 font-medium text-sm rounded-lg border border-gray-300 dark:border-zinc-600 transition-all duration-200 hover:bg-gray-50 dark:hover:bg-zinc-700 hover:border-gray-400 dark:hover:border-zinc-500"
        >
          Cancel
        </button>
        <button
          onclick={confirmSend}
          class="px-5 py-3 bg-gray-800 dark:bg-gray-100 text-gray-100 dark:text-gray-900 font-medium text-sm rounded-lg transition-all duration-200 hover:bg-gray-700 dark:hover:bg-gray-200 active:scale-[0.98] flex items-center justify-center gap-2"
        >
          <Icon icon="carbon:send-filled" class="w-4 h-4" />
          <span>Send</span>
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style lang="postcss">
  .whitespace-pre-wrap {
    white-space: pre-wrap;
  }
</style>