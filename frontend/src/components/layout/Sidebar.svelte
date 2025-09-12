<script>
  import Icon from "@iconify/svelte";
  import ConversationList from "../conversation/ConversationListOptimized.svelte";

  let { onSimCardClick = () => {}, onLogoutClick = () => {} } = $props();
</script>

<div
  class="w-1/5 bg-gradient-to-br from-gray-50 to-gray-100 dark:from-zinc-800 dark:to-zinc-900 p-3 border-r border-gray-200/60 dark:border-zinc-700/60 flex flex-col h-screen backdrop-blur-sm"
>
  <div class="flex-1 overflow-hidden">
    <ConversationList />
  </div>

  <!-- Divider -->
  <div class="my-3 border-t border-gray-300/40 dark:border-zinc-600/40"></div>

  <div
    class="flex flex-col gap-3 p-3 bg-white/60 dark:bg-zinc-900/60 backdrop-blur-sm rounded-xl border border-gray-200/60 dark:border-zinc-700/50 shadow-sm"
  >
    <!-- SIM Cards Button -->
    <button
      class="group w-full flex justify-start items-center gap-3 bg-gradient-to-r from-slate-50 to-gray-50 hover:from-slate-100 hover:to-gray-100
             dark:from-slate-800/40 dark:to-zinc-800/40 dark:hover:from-slate-700/50 dark:hover:to-zinc-700/50
             rounded-lg p-3 transition-all duration-300 ease-out transform hover:scale-[1.02] hover:shadow-lg hover:-translate-y-0.5
             border border-slate-200/40 dark:border-slate-700/40 hover:border-slate-300/60 dark:hover:border-slate-600/60
             active:scale-[0.98] active:translate-y-0"
      onclick={() => onSimCardClick()}
    >
      <div
        class="p-2 bg-slate-100 dark:bg-slate-700/60 rounded-lg group-hover:bg-slate-200 dark:group-hover:bg-slate-600/70 transition-colors duration-200"
      >
        <Icon
          icon="mage:memory-card-fill"
          class="text-slate-600 dark:text-slate-300 w-5 h-5"
        />
      </div>
      <div class="flex flex-col items-start">
        <span class="text-sm font-medium text-slate-700 dark:text-slate-200"
          >SIM Cards</span
        >
        <span class="text-xs text-slate-500 dark:text-slate-400"
          >管理设备信息</span
        >
      </div>
      <Icon
        icon="mage:chevron-right"
        class="text-slate-400 dark:text-slate-500 w-4 h-4 ml-auto opacity-0 group-hover:opacity-100 group-hover:translate-x-1 transition-all duration-300"
      />
    </button>

    <!-- Logout Button -->
    <button
      class="group w-full flex justify-start items-center gap-3 bg-gradient-to-r from-gray-50 to-stone-50 hover:from-gray-100 hover:to-stone-100
             dark:from-stone-800/40 dark:to-gray-800/40 dark:hover:from-stone-700/50 dark:hover:to-gray-700/50
             rounded-lg p-3 transition-all duration-300 ease-out transform hover:scale-[1.02] hover:shadow-lg hover:-translate-y-0.5
             border border-stone-200/40 dark:border-stone-700/40 hover:border-stone-300/60 dark:hover:border-stone-600/60
             active:scale-[0.98] active:translate-y-0"
      onclick={() => onLogoutClick()}
    >
      <div
        class="p-2 bg-stone-100 dark:bg-stone-700/60 rounded-lg group-hover:bg-stone-200 dark:group-hover:bg-stone-600/70 transition-colors duration-200"
      >
        <Icon
          icon="mage:logout"
          class="text-stone-600 dark:text-stone-300 w-5 h-5"
        />
      </div>
      <div class="flex flex-col items-start">
        <span class="text-sm font-medium text-stone-700 dark:text-stone-200"
          >退出登录</span
        >
        <span class="text-xs text-stone-500 dark:text-stone-400"
          >安全退出系统</span
        >
      </div>
      <Icon
        icon="mage:chevron-right"
        class="text-stone-400 dark:text-stone-500 w-4 h-4 ml-auto opacity-0 group-hover:opacity-100 group-hover:translate-x-1 transition-all duration-300"
      />
    </button>
  </div>
</div>
