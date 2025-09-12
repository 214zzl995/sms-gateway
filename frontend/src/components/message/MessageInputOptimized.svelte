<script>
  import Icon from "@iconify/svelte";
  import SimSelector from "./SimSelector.svelte"; // 假设 SimSelector.svelte 在同一目录

  let {
    sendMessageContent = $bindable(""),
    showNewMessage = false,
    concatInputText = "",
    onSend = () => {},
  } = $props();

  let isComposing = $state(false);
  let selectedSim = $state(null); // 与 SimSelector 双向绑定

  function handleSendClick() {
    if (showNewMessage && !concatInputText.trim()) {
      return;
    }

    if (selectedSim) {
      onSend(selectedSim.id);
    }
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && !isComposing) {
      handleSendClick();
    }
  }
</script>

<div
  class="absolute bottom-0 left-0 right-0 h-20 bg-white/95 dark:bg-zinc-900/95 backdrop-blur-xl border-t border-gray-200/50 dark:border-zinc-800 z-10"
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
            class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400 dark:text-gray-500 pointer-events-none"
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
            class="w-full h-12 pl-11 pr-4 bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 rounded-xl text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 transition-all duration-200 outline-none focus:border-gray-400 dark:focus:border-zinc-600 hover:border-gray-300 dark:hover:border-zinc-600"
          />
        </div>
      </div>
    </div>

    <div class="h-6 border-l border-gray-200 dark:border-zinc-700"></div>

    <SimSelector bind:selectedSim />

          <div class="h-6 border-l border-gray-200 dark:border-zinc-700"></div>

    <button
      onclick={handleSendClick}
      disabled={showNewMessage && !concatInputText.trim()}
      class="flex items-center gap-2 px-5 h-12 bg-gray-50 dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 text-gray-700 dark:text-gray-300 font-medium text-sm rounded-xl transition-all duration-200 {showNewMessage &&
      !concatInputText.trim()
        ? 'opacity-40 cursor-not-allowed'
        : 'hover:bg-gray-100 dark:hover:bg-zinc-700 hover:border-gray-300 dark:hover:border-zinc-600 active:bg-gray-200 dark:active:bg-zinc-600'}"
    >
      <Icon
        icon="carbon:send-filled"
        class="w-5 h-5 {showNewMessage && !concatInputText.trim()
          ? 'text-gray-400 dark:text-gray-500'
          : 'text-gray-600 dark:text-gray-400'}"
      />
      <span>Send</span>
    </button>
  </div>
</div>

<style lang="postcss">
  /* 如果有全局样式，可以在这里添加 */
</style>
