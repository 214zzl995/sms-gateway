<script>
  import Icon from "@iconify/svelte";
  import ConversationList from "../conversation/ConversationList.svelte";

  let { onSimCardClick = () => {}, onLogoutClick = () => {} } = $props();
</script>

<div
  class="w-1/5 bg-white dark:bg-zinc-900 p-4 border-r border-gray-200 dark:border-zinc-700 flex flex-col h-screen"
>
  <div class="flex-1 overflow-hidden">
    <ConversationList />
  </div>

  <!-- Divider -->
  <div class="my-4 border-t border-gray-200 dark:border-zinc-700"></div>

  <div class="flex flex-col gap-3">
    <!-- SIM Cards Button -->
    <button
      class="group w-full flex items-center gap-3 p-3 rounded-lg border border-gray-200 dark:border-zinc-700
             bg-white dark:bg-zinc-900 hover:bg-gray-50 dark:hover:bg-zinc-800
             transition-all duration-200 active:scale-[0.98]"
      onclick={() => onSimCardClick()}
    >
      <div class="w-10 h-10 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center">
        <Icon
          icon="carbon:sim-card"
          class="w-5 h-5 text-gray-100 dark:text-gray-900"
        />
      </div>
      <div class="flex flex-col items-start flex-1">
        <span class="text-sm font-semibold text-gray-800 dark:text-gray-100">
          SIM Cards
        </span>
        <span class="text-xs text-gray-500 dark:text-gray-400">
          Manage Device Info
        </span>
      </div>
      <Icon
        icon="carbon:chevron-right"
        class="w-4 h-4 text-gray-400 dark:text-gray-500 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors"
      />
    </button>

    <!-- Logout Button -->
    <button
      class="group w-full flex items-center gap-3 p-3 rounded-lg border border-gray-200 dark:border-zinc-700
             bg-white dark:bg-zinc-900 hover:bg-gray-50 dark:hover:bg-zinc-800
             transition-all duration-200 active:scale-[0.98]"
      onclick={() => onLogoutClick()}
    >
      <div class="w-10 h-10 bg-gray-100 dark:bg-zinc-800 rounded-lg flex items-center justify-center border border-gray-200 dark:border-zinc-700">
        <Icon
          icon="carbon:logout"
          class="w-5 h-5 text-gray-600 dark:text-gray-400"
        />
      </div>
      <div class="flex flex-col items-start flex-1">
        <span class="text-sm font-semibold text-gray-800 dark:text-gray-100">
          Logout
        </span>
        <span class="text-xs text-gray-500 dark:text-gray-400">
          Safe Logout
        </span>
      </div>
      <Icon
        icon="carbon:chevron-right"
        class="w-4 h-4 text-gray-400 dark:text-gray-500 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors"
      />
    </button>
  </div>
</div>
