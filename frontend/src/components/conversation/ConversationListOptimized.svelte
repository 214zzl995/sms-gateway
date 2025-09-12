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

  // Get avatar color based on contact name - simple gray tones
  function getAvatarColor(name) {
    const colors = [
      'bg-gray-400',
      'bg-gray-500',
      'bg-gray-600',
      'bg-zinc-400',
      'bg-zinc-500',
      'bg-zinc-600',
      'bg-slate-400',
      'bg-slate-500'
    ];
    const hash = name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
    return colors[hash % colors.length];
  }

  // Get initials from name
  function getInitials(name) {
    return name.split(' ').map(word => word.charAt(0)).join('').toUpperCase().slice(0, 2);
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header with Search and New Message -->
  <div class="flex flex-col gap-2 mb-3">
    <div class="flex items-center justify-between">
      <h2 class="text-base font-semibold text-gray-700 dark:text-gray-200">消息</h2>
      <div class="flex items-center gap-1">
        <button
          class="p-1.5 rounded-lg bg-gray-100 hover:bg-gray-200 dark:bg-zinc-800 dark:hover:bg-zinc-700
                 text-gray-600 dark:text-gray-400 transition-colors duration-200 focus:outline-none"
          onclick={createNewMessage}
          title="新建消息"
        >
          <Icon icon="mage:edit" class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Search Input -->
    <div class="relative">
      <div
        class="flex items-center gap-2 p-2 rounded-lg border border-gray-200 dark:border-zinc-700
               bg-white dark:bg-zinc-900 transition-colors duration-200
               {searchTemporaryIsActive 
                 ? 'border-gray-400 dark:border-zinc-600' 
                 : 'hover:border-gray-300 dark:hover:border-zinc-600'}"
      >
        <Icon 
          icon="mage:search" 
          class="w-3.5 h-3.5 text-gray-400 dark:text-zinc-500" 
        />
        <input
          type="text"
          onfocus={searchHandleFocus}
          onblur={searchHandleBlur}
          bind:value={searchTemporaryValue}
          class="flex-1 bg-transparent border-0 outline-none focus:outline-none text-sm text-gray-600 dark:text-gray-300
                 placeholder-gray-400 dark:placeholder-zinc-500"
          placeholder="搜索联系人..."
        />
        {#if searchTemporaryValue}
          <button
            onclick={() => searchTemporaryValue = ""}
            class="p-1 rounded-full hover:bg-gray-100 dark:hover:bg-zinc-800 transition-colors focus:outline-none"
          >
            <Icon icon="mage:multiply" class="w-3 h-3 text-gray-400" />
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Conversations List -->
  <div class="flex-1 overflow-hidden">
    <div class="h-full overflow-y-auto scrollbar-hide">
      <div class="space-y-1">
        {#if $conversationLoading}
          <!-- Loading skeleton -->
          {#each Array(5) as _}
            <div class="flex items-center gap-2.5 p-2.5 rounded-lg animate-pulse">
              <div class="w-8 h-8 bg-gray-200 dark:bg-zinc-700 rounded-full"></div>
              <div class="flex-1 space-y-1.5">
                <div class="h-3 bg-gray-200 dark:bg-zinc-700 rounded w-3/4"></div>
                <div class="h-2.5 bg-gray-200 dark:bg-zinc-700 rounded w-1/2"></div>
              </div>
              <div class="w-10 h-2.5 bg-gray-200 dark:bg-zinc-700 rounded"></div>
            </div>
          {/each}
        {:else}
          {#each filteredConversations as conversation (conversation.contact.id)}
            <div
              animate:flip={{ duration: 300, easing: cubicOut }}
              transition:fade={{ duration: 200 }}
              class="conversation-item relative group cursor-pointer focus:outline-none"
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
                class="flex items-center gap-2.5 p-2.5 rounded-lg transition-colors duration-200 border
                       {$currentContact?.id === conversation.contact.id
                         ? 'bg-gray-200 dark:bg-zinc-700 border-gray-300 dark:border-zinc-600'
                         : 'border-transparent hover:bg-gray-50 dark:hover:bg-zinc-800/50 hover:border-gray-200 dark:hover:border-zinc-700'}"
              >
                <!-- Avatar -->
                <div class="relative flex-shrink-0">
                  <div class="w-8 h-8 rounded-full flex items-center justify-center text-white font-medium text-xs
                              {getAvatarColor(conversation.contact.name)}">
                    {getInitials(conversation.contact.name)}
                  </div>
                  
                  <!-- Unread indicator -->
                  {#if conversation.sms_preview?.status === SmsStatus.Unread}
                    <div 
                      class="absolute -top-0.5 -right-0.5 w-3 h-3 bg-red-500 rounded-full
                             border border-white dark:border-zinc-900"
                      transition:scale={{ duration: 200 }}
                    >
                    </div>
                  {/if}
                </div>

                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <!-- Name and Time -->
                  <div class="flex items-center justify-between mb-0.5">
                    <h3 class="font-medium text-sm text-gray-700 dark:text-gray-200 truncate pr-2">
                      {conversation.contact.name}
                    </h3>
                    {#if !conversation.contact.new}
                      <div class="text-xs text-gray-400 dark:text-zinc-500 flex-shrink-0">
                        {#if conversation.sms_preview?.timestamp}
                          <span>{formatDate(conversation.sms_preview.timestamp)}</span>
                        {:else}
                          <span class="text-gray-500 dark:text-gray-400">新建</span>
                        {/if}
                      </div>
                    {/if}
                  </div>

                  <!-- Preview Message and Time for new items -->
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-1.5 flex-1 min-w-0">
                      {#if conversation.sms_preview}
                        <!-- SIM Card Badge -->
                        <div class="flex-shrink-0 flex items-center">
                          <span 
                            class="inline-flex items-center px-2 py-1 text-xs rounded-md font-medium shadow-sm transition-colors duration-300 leading-none
                                   {$currentContact?.id === conversation.contact.id
                                     ? 'bg-gray-400 dark:bg-zinc-600 text-white'
                                     : 'bg-gray-200 dark:bg-zinc-700 text-gray-600 dark:text-zinc-400'}"
                          >
                            {getSimCardDisplayName(conversation.sms_preview.sim_id)}
                          </span>
                        </div>
                      {/if}
                      
                      <!-- Message Content -->
                      <div class="flex-1 min-w-0">
                        <p class="text-xs text-gray-500 dark:text-zinc-500 line-clamp-1">
                          {#if conversation.sms_preview}
                            {conversation.sms_preview.message}
                          {:else}
                            <span class="italic text-gray-400 dark:text-gray-500">
                              点击开始对话
                            </span>
                          {/if}
                        </p>
                      </div>
                    </div>
                    
                    <!-- Time and Delete button for new items -->
                    <div class="flex items-center gap-1.5 flex-shrink-0">
                      {#if conversation.contact.new}
                        <div class="text-xs text-gray-400 dark:text-zinc-500">
                          <span class="text-gray-500 dark:text-gray-400">新建</span>
                        </div>
                      {/if}
                    </div>
                  </div>
                </div>

                <!-- Delete button for new contacts -->
                {#if conversation.contact.new === true}
                  <div class="absolute top-2.5 right-2.5">
                    <button
                      class="w-4 h-4 rounded-full bg-gray-300 hover:bg-gray-400 dark:bg-zinc-600 dark:hover:bg-zinc-500
                             text-gray-600 dark:text-gray-300 transition-colors duration-200 focus:outline-none flex items-center justify-center"
                      onclick={(e) => {
                        e.stopPropagation();
                        deleteConversationHandleClick(conversation);
                      }}
                      title="删除会话"
                    >
                      <Icon icon="mage:multiply" class="w-2.5 h-2.5" />
                    </button>
                  </div>
                {/if}

              </div>
            </div>
          {/each}
        {/if}

        <!-- Empty State -->
        {#if !$conversationLoading && filteredConversations.length === 0}
          <div class="flex flex-col items-center justify-center py-8 text-center">
            <div class="w-12 h-12 rounded-full bg-gray-100 dark:bg-zinc-800 flex items-center justify-center mb-3">
              <Icon icon="mage:message-dots" class="w-6 h-6 text-gray-400 dark:text-zinc-600" />
            </div>
            <p class="text-gray-600 dark:text-zinc-400 text-sm mb-1">
              {searchTemporaryValue ? '没有找到相关对话' : '暂无对话'}
            </p>
            <p class="text-xs text-gray-400 dark:text-zinc-500 mb-3">
              {searchTemporaryValue ? '尝试修改搜索条件' : '点击上方按钮创建新对话'}
            </p>
            {#if !searchTemporaryValue}
              <button
                onclick={createNewMessage}
                class="px-3 py-1.5 bg-gray-600 hover:bg-gray-700 text-white rounded-lg text-sm
                       transition-colors duration-200 flex items-center gap-1.5 focus:outline-none"
              >
                <Icon icon="mage:plus" class="w-3 h-3" />
                新建对话
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .scrollbar-hide {
    scrollbar-width: none;
    -ms-overflow-style: none;
  }
  
  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }

  .line-clamp-1 {
    display: -webkit-box;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>