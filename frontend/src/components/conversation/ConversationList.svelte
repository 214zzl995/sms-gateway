<script>
  import Icon from "@iconify/svelte";
  import { formatDate } from "../../js/dateFormat";
  import { flip } from "svelte/animate";
  import { fade, slide, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import {
    conversations,
    currentContact,
    changeCurrentConversation,
    conversationLoading,
    deleteConversation,
  } from "../../stores/conversation";
  import { generateUUID } from "../../js/uuid";
  import { simCards } from "../../stores/simcards";

  const SmsStatus = {
    Unread: 0,
    Read: 1,
    Loading: 2,
    Failed: 3,
  };

  let searchTemporaryIsActive = $state(false);
  let searchTemporaryValue = $state("");

  // Function to get SIM card alias by ID
  function getSimCardDisplayName(simId) {
    if (!simId) {
      return "Unknown";
    }
    
    const simCard = $simCards.find(sim => sim.id === simId);
    if (simCard) {
      return simCard.alias || simCard.phone_number || `SIM ${simId.slice(-8)}`;
    }
    return `SIM ${simId.slice(-8)}`;
  }

  let filteredConversations = $derived(
    $conversationLoading
      ? []
      : $conversations.filter(
          (conversation) =>
            searchTemporaryValue === "" ||
            conversation.contact.name
              .toLowerCase()
              .includes(searchTemporaryValue.toLowerCase())
        )
  );

  function searchHandleFocus() {
    searchTemporaryIsActive = true;
  }

  function searchHandleBlur() {
    searchTemporaryIsActive = false;
  }

  function conversationHandleClick(conversation) {
    changeCurrentConversation(conversation.contact);
  }

  function createNewMessage() {
    // Check if there's already a new message item
    const existingNewMessage = $conversations.find(conv => conv.contact.new === true);
    
    if (existingNewMessage) {
      // Switch to the existing new message
      changeCurrentConversation(existingNewMessage.contact);
      return;
    }

    // Create new message item
    const uuid = generateUUID();
    changeCurrentConversation({
      id: uuid,
      name: "新信息",
      new: true,
    });
  }

  function deleteConversationHandleClick(conversation) {
    if ($conversations.length === 1) {
      return;
    }
    deleteConversation(conversation.contact.id);
    changeCurrentConversation($conversations[0]?.contact);
  }

  // Get avatar with initials
  function getInitials(name) {
    return name.split(' ').map(word => word.charAt(0)).join('').toUpperCase().slice(0, 2);
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header with Search and New Message -->
  <div class="flex flex-col gap-3 mb-4">
    <div class="flex items-center justify-between">
      <h2 class="text-base font-semibold text-gray-800 dark:text-gray-100">Messages</h2>
      <button
        class="p-2 rounded-lg bg-gray-800 dark:bg-gray-100 text-gray-100 dark:text-gray-900
               hover:bg-gray-700 dark:hover:bg-gray-200 transition-all duration-200 active:scale-[0.95]"
        onclick={createNewMessage}
        title="New Message"
      >
        <Icon icon="carbon:add" class="w-4 h-4" />
      </button>
    </div>

    <!-- Search Input -->
    <div class="relative">
      <div
        class="flex items-center gap-2.5 px-3 py-2 rounded-lg border transition-all duration-200
               bg-white dark:bg-zinc-900
               {searchTemporaryIsActive 
                 ? 'border-gray-400 dark:border-zinc-500' 
                 : 'border-gray-300 dark:border-zinc-600 hover:border-gray-400 dark:hover:border-zinc-500'}"
      >
        <Icon 
          icon="carbon:search" 
          class="w-4 h-4 text-gray-500 dark:text-gray-400" 
        />
        <input
          type="text"
          onfocus={searchHandleFocus}
          onblur={searchHandleBlur}
          bind:value={searchTemporaryValue}
          class="flex-1 bg-transparent border-0 outline-none text-sm text-gray-700 dark:text-gray-200
                 placeholder-gray-400 dark:placeholder-gray-500"
          placeholder="Search conversations..."
        />
        {#if searchTemporaryValue}
          <button
            onclick={() => searchTemporaryValue = ""}
            class="p-0.5 rounded hover:bg-gray-100 dark:hover:bg-zinc-800 transition-colors"
          >
            <Icon icon="carbon:close" class="w-3.5 h-3.5 text-gray-500 dark:text-gray-400" />
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Conversations List -->
  <div class="flex-1 overflow-hidden">
    <div class="h-full overflow-y-auto scrollbar-thin">
      <div class="space-y-1">
        {#if $conversationLoading}
          <!-- Loading skeleton -->
          {#each Array(5) as _}
            <div class="flex items-center gap-3 p-3 rounded-lg animate-pulse">
              <div class="w-10 h-10 bg-gray-200 dark:bg-zinc-700 rounded-lg"></div>
              <div class="flex-1 space-y-2">
                <div class="h-3 bg-gray-200 dark:bg-zinc-700 rounded w-3/4"></div>
                <div class="h-2.5 bg-gray-200 dark:bg-zinc-700 rounded w-1/2"></div>
              </div>
            </div>
          {/each}
        {:else}
          {#each filteredConversations as conversation (conversation.contact.id)}
            <div
              animate:flip={{ duration: 300, easing: cubicOut }}
              transition:fade={{ duration: 200 }}
              class="relative group cursor-pointer focus:outline-none"
              role="button"
              tabindex="0"
              onclick={() => conversationHandleClick(conversation)}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  conversationHandleClick(conversation);
                }
              }}
            >
              <div
                class="flex items-center gap-3 p-3 rounded-lg transition-all duration-200 border
                       {$currentContact?.id === conversation.contact.id
                         ? 'bg-gray-100 dark:bg-zinc-800 border-gray-300 dark:border-zinc-600'
                         : 'border-transparent hover:bg-gray-50 dark:hover:bg-zinc-800/50'}"
              >
                <!-- Avatar -->
                <div class="relative flex-shrink-0">
                  <div class="w-10 h-10 rounded-lg bg-gray-800 dark:bg-gray-200 flex items-center justify-center">
                    <span class="text-sm font-medium text-gray-100 dark:text-gray-800">
                      {getInitials(conversation.contact.name)}
                    </span>
                  </div>
                  
                  <!-- Unread indicator -->
                  {#if conversation.sms_preview?.status === SmsStatus.Unread}
                    <div 
                      class="absolute -top-1 -right-1 w-2.5 h-2.5 bg-red-500 rounded-full
                             border-2 border-white dark:border-zinc-900"
                      transition:scale={{ duration: 200 }}
                    ></div>
                  {/if}
                </div>

                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <!-- Name and Time -->
                  <div class="flex items-center justify-between mb-1">
                    <h3 class="font-medium text-sm text-gray-800 dark:text-gray-100 truncate pr-2">
                      {conversation.contact.name}
                    </h3>
                    {#if !conversation.contact.new && conversation.sms_preview?.timestamp}
                      <span class="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0">
                        {formatDate(conversation.sms_preview.timestamp)}
                      </span>
                    {/if}
                  </div>

                  <!-- Preview Message and SIM Badge -->
                  <div class="flex items-center gap-2">
                    {#if conversation.sms_preview}
                      <!-- SIM Card Badge -->
                      <span 
                        class="inline-flex items-center px-1.5 py-0.5 text-[10px] rounded font-medium
                               bg-gray-200 dark:bg-zinc-700 text-gray-600 dark:text-gray-400
                               flex-shrink-0"
                      >
                        {getSimCardDisplayName(conversation.sms_preview.sim_id)}
                      </span>
                      
                      <!-- Message Content -->
                      <p class="text-xs text-gray-500 dark:text-gray-400 line-clamp-1 flex-1">
                        {conversation.sms_preview.message}
                      </p>
                    {:else}
                      <span class="text-xs text-gray-400 dark:text-gray-500 italic">
                        {conversation.contact.new ? 'New conversation' : 'No messages'}
                      </span>
                    {/if}
                  </div>
                </div>

                <!-- Delete button for new contacts -->
                {#if conversation.contact.new === true}
                  <button
                    class="opacity-0 group-hover:opacity-100 p-1.5 rounded-lg
                           bg-gray-200 dark:bg-zinc-700 hover:bg-gray-300 dark:hover:bg-zinc-600
                           text-gray-600 dark:text-gray-400 transition-all duration-200"
                    onclick={(e) => {
                      e.stopPropagation();
                      deleteConversationHandleClick(conversation);
                    }}
                    title="Delete"
                  >
                    <Icon icon="carbon:trash-can" class="w-3.5 h-3.5" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        {/if}

        <!-- Empty State -->
        {#if !$conversationLoading && filteredConversations.length === 0}
          <div class="flex flex-col items-center justify-center py-12 text-center">
            <div class="w-16 h-16 rounded-lg bg-gray-100 dark:bg-zinc-800 flex items-center justify-center mb-4">
              <Icon icon="carbon:chat" class="w-8 h-8 text-gray-400 dark:text-gray-600" />
            </div>
            <p class="text-gray-700 dark:text-gray-300 text-sm font-medium mb-1">
              {searchTemporaryValue ? 'No results found' : 'No conversations'}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-500 mb-4">
              {searchTemporaryValue ? 'Try a different search term' : 'Start a new conversation to get started'}
            </p>
            {#if !searchTemporaryValue}
              <button
                onclick={createNewMessage}
                class="px-4 py-2 bg-gray-800 dark:bg-gray-100 text-gray-100 dark:text-gray-900
                       hover:bg-gray-700 dark:hover:bg-gray-200 rounded-lg text-sm font-medium
                       transition-all duration-200 flex items-center gap-2 active:scale-[0.95]"
              >
                <Icon icon="carbon:add" class="w-4 h-4" />
                New Conversation
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: rgb(156 163 175) transparent;
  }
  
  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
  }
  
  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgb(156 163 175);
    border-radius: 3px;
  }
  

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background-color: rgb(107 114 128);
  }

  .line-clamp-1 {
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>