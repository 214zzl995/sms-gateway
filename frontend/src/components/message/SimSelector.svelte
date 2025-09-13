<script>
  import Icon from "@iconify/svelte";
  import { simCards } from "../../stores/simcards";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  let { selectedSim = $bindable(null) } = $props();

  let showSimSelector = $state(false);
  let searchText = $state("");
  let expandedContent = $state(false);

  let filteredSimCards = $derived(
    $simCards.filter(
      (sim) =>
        !searchText ||
        (sim.alias &&
          sim.alias.toLowerCase().includes(searchText.toLowerCase())) ||
        (sim.phone_number && sim.phone_number.includes(searchText)) ||
        sim.id.toLowerCase().includes(searchText.toLowerCase()),
    ),
  );

  $effect(() => {
    const storedSimId = localStorage.getItem("selectedSimId");
    if (storedSimId && $simCards.length > 0) {
      const foundSim = $simCards.find((sim) => sim.id === storedSimId);
      selectedSim = foundSim || $simCards[0];
    } else if (!selectedSim && $simCards.length > 0) {
      selectedSim = $simCards[0];
    }
  });

  function selectSim(sim) {
    selectedSim = sim;
    localStorage.setItem("selectedSimId", sim.id);
    expandedContent = false;
    setTimeout(() => {
      showSimSelector = false;
    }, 200);
    searchText = "";
  }

  function toggleSimSelector() {
    if (!showSimSelector) {
      showSimSelector = true;
      requestAnimationFrame(() => {
        expandedContent = true;
      });
    } else {
      expandedContent = false;
      setTimeout(() => {
        showSimSelector = false;
      }, 200);
    }
    searchText = "";
  }

  function clickOutside(node) {
    const handleClick = (event) => {
      if (!node.contains(event.target)) {
        expandedContent = false;
        setTimeout(() => {
          showSimSelector = false;
        }, 200);
        searchText = "";
      }
    };
    document.addEventListener("click", handleClick, true);

    return {
      destroy() {
        document.removeEventListener("click", handleClick, true);
      },
    };
  }
</script>

<div class="relative" use:clickOutside>
  <div
    class="relative transition-all duration-300 ease-out"
    style="width: {showSimSelector ? '320px' : '180px'};"
  >
    {#if expandedContent}
      <!-- 向上弹出的内容 -->
      <div
        class="absolute bottom-full left-0 right-0 overflow-y-hide scrollbar-hide"
        transition:fly={{ y: 100, duration: 200, easing: quintOut }}
      >
        <div
          class="bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 border-b-0 rounded-t-xl overflow-hidden shadow-lg"
        >
          <!-- 标题栏 -->
          <div
            class="px-4 py-3 bg-gray-50 dark:bg-zinc-800 border-b border-gray-200 dark:border-zinc-700"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-5 h-5 bg-gray-900 dark:bg-gray-100 rounded flex items-center justify-center">
                  <Icon
                    icon="carbon:sim-card"
                    class="w-3 h-3 text-gray-100 dark:text-gray-900"
                  />
                </div>
                <span
                  class="text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-widest"
                >
                  Select SIM Card
                </span>
              </div>
              <button
                onclick={toggleSimSelector}
                class="p-1 hover:bg-gray-200 dark:hover:bg-zinc-700 rounded-md transition-colors"
              >
                <Icon
                  icon="carbon:close"
                  class="w-3.5 h-3.5 text-gray-500 dark:text-gray-400"
                />
              </button>
            </div>
          </div>

          <!-- 搜索框 -->
          <div
            class="px-3 py-3 bg-gray-50/50 dark:bg-zinc-800/50 border-b border-gray-200 dark:border-zinc-700"
          >
            <div class="relative">
              <Icon
                icon="carbon:search"
                class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-gray-500"
              />
              <input
                type="text"
                bind:value={searchText}
                placeholder="Search by name or number..."
                class="w-full pl-9 pr-3 py-2 bg-white dark:bg-zinc-900 border border-gray-300 dark:border-zinc-600 rounded-lg text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 outline-none focus:border-gray-500 dark:focus:border-zinc-500 hover:border-gray-400 dark:hover:border-zinc-600 transition-all duration-200"
              />
            </div>
          </div>

          <!-- SIM 卡列表 -->
          <div class="max-h-56 overflow-y-auto scrollbar-thin">
            <div class="py-1">
              {#each filteredSimCards as sim (sim.id)}
                {@const isSelected = sim.id === selectedSim?.id}
                <button
                  onclick={() => selectSim(sim)}
                  class="w-full flex items-center gap-3 px-3 py-2.5 transition-all duration-150 {isSelected
                    ? 'bg-gray-100 dark:bg-zinc-800'
                    : 'hover:bg-gray-50 dark:hover:bg-zinc-800/50'}"
                >
                  <div
                    class="flex-shrink-0 w-9 h-9 rounded-lg flex items-center justify-center transition-colors {isSelected
                      ? 'bg-gray-900 dark:bg-gray-100'
                      : 'bg-gray-100 dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700'}"
                  >
                    <Icon
                      icon={isSelected ? "carbon:checkmark-filled" : "carbon:sim-card"}
                      class="w-4 h-4 {isSelected
                        ? 'text-gray-100 dark:text-gray-900'
                        : 'text-gray-600 dark:text-gray-400'}"
                    />
                  </div>
                  <div class="flex-1 text-left">
                    <div
                      class="text-sm font-medium {isSelected
                        ? 'text-gray-900 dark:text-gray-100'
                        : 'text-gray-700 dark:text-gray-300'}"
                    >
                      {sim.alias ||
                        sim.phone_number ||
                        `SIM ${sim.id.slice(-6)}`}
                    </div>
                    {#if sim.alias && sim.phone_number}
                      <div
                        class="text-xs {isSelected
                          ? 'text-gray-600 dark:text-gray-400'
                          : 'text-gray-500 dark:text-gray-500'} font-mono"
                      >
                        {sim.phone_number}
                      </div>
                    {/if}
                  </div>
                  {#if isSelected}
                    <div class="flex items-center gap-1.5">
                      <div class="w-1.5 h-1.5 bg-green-500 rounded-full"></div>
                      <span
                        class="text-xs font-medium text-gray-600 dark:text-gray-400"
                      >
                        Active
                      </span>
                    </div>
                  {/if}
                </button>
              {:else}
                <div
                  class="py-8 text-center text-sm text-gray-400 dark:text-gray-500"
                >
                  <Icon
                    icon="carbon:search-locate-mirror"
                    class="w-8 h-8 mx-auto mb-2 opacity-50"
                  />
                  No SIM cards found
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- 主按钮 - 始终显示，根据状态改变样式 -->
    <div
      class="relative bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 transition-all duration-250 ease-out {expandedContent
        ? 'rounded-b-xl border-t-0 shadow-lg'
        : 'rounded-xl hover:shadow-sm'}"
    >
      {#if expandedContent}
        <!-- 连接处的填充块 -->
        <div
          class="absolute top-0 left-0 right-0 h-1 bg-white dark:bg-zinc-900 -mt-[1px]"
        ></div>
      {/if}

      <button
        onclick={toggleSimSelector}
        class="w-full flex items-center gap-2.5 px-4 h-12 hover:bg-gray-50 dark:hover:bg-zinc-800 transition-colors duration-200 {expandedContent
          ? 'rounded-b-xl'
          : 'rounded-xl'}"
      >
        <div class="w-5 h-5 bg-gray-900 dark:bg-gray-100 rounded flex items-center justify-center flex-shrink-0">
          <Icon
            icon="carbon:sim-card"
            class="w-3 h-3 text-gray-100 dark:text-gray-900"
          />
        </div>
        {#if selectedSim}
          <span
            class="flex-1 text-left text-sm font-medium text-gray-800 dark:text-gray-200 truncate"
          >
            {selectedSim.alias ||
              selectedSim.phone_number ||
              `SIM ${selectedSim.id.slice(-6)}`}
          </span>
          <div class="flex items-center gap-1.5">
            <div class="w-1.5 h-1.5 bg-green-500 rounded-full"></div>
           
          </div>
        {:else}
          <span
            class="flex-1 text-left text-sm text-gray-500 dark:text-gray-400"
          >
            Select SIM
          </span>
        {/if}
        <Icon
          icon="carbon:chevron-down"
          class="w-4 h-4 text-gray-400 dark:text-gray-500 transition-transform duration-300 flex-shrink-0"
          style="transform: rotate({expandedContent ? '180deg' : '0deg'})"
        />
      </button>
    </div>
  </div>
</div>

<style lang="postcss">
  /* Refined scrollbar */
  .scrollbar-thin::-webkit-scrollbar {
    width: 4px;
  }

  .scrollbar-thin::-webkit-scrollbar-track {
    @apply bg-gray-100 dark:bg-zinc-800;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    @apply bg-gray-400 dark:bg-zinc-600 rounded-full;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    @apply bg-gray-500 dark:bg-zinc-500;
  }

  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }

  .scrollbar-hide {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>