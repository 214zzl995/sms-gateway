<script>
  import Icon from "@iconify/svelte";
  import { simCards } from "../../stores/simcards";
  import { fade, slide } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  let {
    sendMessageContent = $bindable(""),
    showNewMessage = false,
    concatInputText = "",
    onSend = () => {},
  } = $props();

  let showSimSelector = $state(false);
  let selectedSim = $state(null);
  let searchText = $state("");
  let isComposing = $state(false);
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
    if (!selectedSim && $simCards.length > 0) {
      selectedSim = $simCards[0];
    }
  });

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

  function toggleSimSelector() {
    if (!showSimSelector) {
      showSimSelector = true;
      // Delay content expansion to create smooth animation
      setTimeout(() => {
        expandedContent = true;
      }, 150);
    } else {
      expandedContent = false;
      setTimeout(() => {
        showSimSelector = false;
      }, 150);
    }
    searchText = "";
  }

  function selectSim(sim) {
    selectedSim = sim;
    expandedContent = false;
    setTimeout(() => {
      showSimSelector = false;
    }, 150);
    searchText = "";
  }

  function clickOutside(node) {
    const handleClick = (event) => {
      if (!node.contains(event.target)) {
        expandedContent = false;
        setTimeout(() => {
          showSimSelector = false;
        }, 150);
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

<div
  class="absolute bottom-0 left-0 right-0 h-20 bg-white/95 dark:bg-zinc-900/95 backdrop-blur-xl border-t border-gray-200/50 dark:border-zinc-800 z-10"
>
  <div class="flex items-center gap-3 h-full px-6 max-w-6xl mx-auto">
    <!-- SIM Card Selector Area -->
    <div class="relative flex-shrink-0" use:clickOutside>
      <!-- Unified Container (Button + Expanded Content) -->
      <!-- Changed positioning to expand upward instead of downward -->
      <div
        class="absolute {expandedContent ? 'bottom-full mb-2' : 'bottom-0'} left-0 bg-gray-50 dark:bg-zinc-800
               border border-gray-200 dark:border-zinc-700
               shadow-sm hover:shadow-md
               transition-all duration-300 ease-out
               {showSimSelector ? 'rounded-xl' : 'rounded-xl'}"
        style="width: {showSimSelector ? '320px' : '180px'}; 
               min-height: 48px;
               {expandedContent ? 'box-shadow: 0 -10px 25px -5px rgba(0, 0, 0, 0.1), 0 -8px 10px -6px rgba(0, 0, 0, 0.1);' : ''}"
      >
        <!-- Expanded Content (Now appears ABOVE the button when expanded) -->
        {#if expandedContent}
          <div
            class="w-full"
            transition:slide={{ duration: 200, easing: quintOut }}
          >
            <!-- Header -->
            <div
              class="px-4 py-2.5 bg-gray-50/50 dark:bg-zinc-800/50 border-b border-gray-100 dark:border-zinc-700 rounded-t-xl"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <Icon
                    icon="carbon:sim-card"
                    class="w-4 h-4 text-gray-500 dark:text-gray-400"
                  />
                  <span
                    class="text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider"
                  >
                    Available SIM Cards
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

            <!-- Search Bar -->
            <div
              class="px-3 py-2.5 border-b border-gray-100 dark:border-zinc-700"
            >
              <div class="relative">
                <Icon
                  icon="carbon:search"
                  class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-gray-500"
                />
                <input
                  type="text"
                  bind:value={searchText}
                  placeholder="Search SIM cards..."
                  class="w-full pl-9 pr-3 py-2 bg-white dark:bg-zinc-900
                         border border-gray-200 dark:border-zinc-600 rounded-lg
                         text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500
                         focus:outline-none focus:ring-1 focus:ring-blue-500/50 focus:border-blue-500
                         transition-all duration-200"
                />
              </div>
            </div>

            <!-- SIM Cards List -->
            <div class="max-h-56 overflow-y-auto">
              <div class="py-1">
                {#each filteredSimCards as sim (sim.id)}
                  {@const isSelected = sim.id === selectedSim?.id}
                  <button
                    onclick={() => selectSim(sim)}
                    class="w-full flex items-center gap-3 px-3 py-2.5
                           transition-all duration-150 group
                           {isSelected
                      ? 'bg-blue-50/50 dark:bg-blue-950/10'
                      : 'hover:bg-gray-100/50 dark:hover:bg-zinc-700/30 cursor-pointer'}"
                  >
                    <div
                      class="flex-shrink-0 p-1.5 rounded-lg
                                {isSelected
                        ? 'bg-blue-100 dark:bg-blue-900/20'
                        : 'bg-gray-100 dark:bg-zinc-700 group-hover:bg-gray-200 dark:group-hover:bg-zinc-600'}
                                transition-colors"
                    >
                      <Icon
                        icon={isSelected
                          ? "carbon:checkmark"
                          : "carbon:sim-card"}
                        class="w-4 h-4 {isSelected
                          ? 'text-blue-600 dark:text-blue-400'
                          : 'text-gray-600 dark:text-gray-400'}"
                      />
                    </div>
                    <div class="flex-1 text-left">
                      <div
                        class="text-sm font-medium {isSelected
                          ? 'text-blue-900 dark:text-blue-100'
                          : 'text-gray-900 dark:text-white'}"
                      >
                        {sim.alias ||
                          sim.phone_number ||
                          `SIM ${sim.id.slice(-6)}`}
                      </div>
                      {#if sim.alias && sim.phone_number}
                        <div
                          class="text-xs {isSelected
                            ? 'text-blue-700 dark:text-blue-300'
                            : 'text-gray-500 dark:text-gray-400'}"
                        >
                          {sim.phone_number}
                        </div>
                      {/if}
                    </div>
                    {#if isSelected}
                      <span
                        class="text-xs font-medium text-blue-600 dark:text-blue-400 px-2 py-0.5 bg-blue-100 dark:bg-blue-900/30 rounded"
                      >
                        Current
                      </span>
                    {/if}
                  </button>
                {:else}
                  <div
                    class="py-8 text-center text-sm text-gray-400 dark:text-gray-500"
                  >
                    No SIM cards found
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {/if}

        <!-- Button Part (Always at the bottom of the container) -->
        <button
          onclick={toggleSimSelector}
          class="w-full flex items-center gap-2 px-4 h-12
                 hover:bg-gray-100 dark:hover:bg-zinc-700/50
                 transition-colors duration-200
                 {expandedContent
            ? 'border-t border-gray-200 dark:border-zinc-700 rounded-b-xl'
            : 'rounded-xl'}"
        >
          <Icon
            icon={expandedContent && selectedSim
              ? "carbon:checkmark-filled"
              : "carbon:sim-card"}
            class="w-5 h-5 {expandedContent
              ? 'text-blue-500'
              : 'text-gray-600 dark:text-gray-400'}"
          />
          {#if selectedSim}
            <span
              class="flex-1 text-left text-sm font-medium text-gray-700 dark:text-gray-300 truncate"
            >
              {selectedSim.alias ||
                selectedSim.phone_number ||
                `SIM ${selectedSim.id.slice(-6)}`}
            </span>
            {#if !expandedContent}
              <div
                class="w-2 h-2 bg-green-500 rounded-full animate-pulse"
              ></div>
            {/if}
          {:else}
            <span
              class="flex-1 text-left text-sm font-medium text-gray-500 dark:text-gray-400"
            >
              Select SIM
            </span>
          {/if}
          <Icon
            icon={expandedContent
              ? "carbon:chevron-down"
              : "carbon:chevron-down"}
            class="w-4 h-4 text-gray-400 dark:text-gray-500 transition-transform duration-300"
            style="transform: rotate({expandedContent ? '180deg' : '0deg'})"
          />
        </button>
      </div>
    </div>

    <!-- Message Input Area -->
    <div class="flex-1">
      <div
        class="relative"
        class:opacity-50={showNewMessage && !concatInputText.trim()}
        class:pointer-events-none={showNewMessage && !concatInputText.trim()}
      >
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
          class="w-full h-12 px-4 bg-gray-50 dark:bg-zinc-800
                 border border-gray-200 dark:border-zinc-700 rounded-xl
                 text-sm text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500
                 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500
                 focus:bg-white dark:focus:bg-zinc-900
                 transition-all duration-200"
        />
      </div>
    </div>

    <!-- Send Button -->
    <button
      onclick={handleSendClick}
      disabled={showNewMessage && !concatInputText.trim()}
      class="flex items-center gap-2 px-5 h-12
             bg-gradient-to-r from-blue-500 to-blue-600
             hover:from-blue-600 hover:to-blue-700
             disabled:from-gray-400 disabled:to-gray-500
             text-white font-medium text-sm rounded-xl
             shadow-lg shadow-blue-500/25 hover:shadow-xl hover:shadow-blue-500/30
             disabled:shadow-none
             transform hover:-translate-y-0.5 active:translate-y-0
             transition-all duration-200
             disabled:cursor-not-allowed disabled:transform-none"
    >
      <Icon icon="carbon:send-filled" class="w-5 h-5" />
      <span>Send</span>
    </button>
  </div>
</div>

<style lang="postcss">
  /* Custom scrollbar for SIM list */
  div::-webkit-scrollbar {
    width: 6px;
  }

  div::-webkit-scrollbar-track {
    @apply bg-transparent;
  }

  div::-webkit-scrollbar-thumb {
    @apply bg-gray-300 dark:bg-zinc-600 rounded-full;
  }

  div::-webkit-scrollbar-thumb:hover {
    @apply bg-gray-400 dark:bg-zinc-500;
  }

  /* Smooth shadow transition */
  @keyframes expandShadow {
    from {
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }
    to {
      box-shadow:
        0 -10px 25px -5px rgba(0, 0, 0, 0.1),
        0 -8px 10px -6px rgba(0, 0, 0, 0.1);
    }
  }
</style>