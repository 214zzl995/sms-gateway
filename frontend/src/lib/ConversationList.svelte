<script>
  import Icon from "@iconify/svelte";
  import { formatDate } from "../js/dateFormat";
  import { flip } from "svelte/animate";
  import { fade } from "svelte/transition";
  import {
    conversations,
    currentContact,
    changeCurrentConversation,
    conversationLoading,
    deleteConversation,
    createNewContactName,
  } from "../stores/conversation";
  import { apiClient } from "../js/api";

  const SmsStatus = {
    Unread: 0,
    Read: 1,
    Loading: 2,
    Failed: 3,
  };

  let searchTemporaryIsActive = $state(false);
  let searchTemporaryValue = $state("");

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

    setTimeout(() => {
      searchTemporaryIsActive = false;
    }, 500);
  }

  /**
   * @param {{ contact: { id: any; }; }} conversation
   */
  function conversationHandleClick(conversation) {
    changeCurrentConversation(conversation.contact);
  }
  function createNewMessage() {
    if ( $conversations[0].contact.new) {
      return;
    }
    apiClient.createContact(createNewContactName()).then((res) => {
      changeCurrentConversation({
        id: res.data,
        name: "新信息",
        new: true,
      });
    });
  }

  /**
   * @param {{ contact: { id: number; }; }} conversation
   */
  function deleteConversationHandleClick(conversation) {
    if ($conversations.length === 1) {
      return;
    }
    apiClient.deleteContactById(conversation.contact.id).then(() => {
      deleteConversation(conversation.contact.id);
      changeCurrentConversation($conversations[0].contact);
    });
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex justify-between items-center mb-2">
    <div
      class="flex items-center border-0 rounded-md px-2 bg-gray-200 dark:bg-zinc-700 transition-colors duration-500 flex-1"
      class:bg-gray-300={searchTemporaryIsActive}
      class:dark:bg-zinc-600={searchTemporaryIsActive}
    >
      <Icon icon="mage:search" class="text-gray-400 text-xl" />
      <input
        type="text"
        onfocus={searchHandleFocus}
        bind:value={searchTemporaryValue}
        class="w-full ml-2 p-1 rounded-md border-0 focus:outline-none focus:ring-0 bg-transparent"
        placeholder="Search"
      />
    </div>

    <button
      class="text-stone-800 text-xl p-2 rounded-full bg-gray-200 dark:bg-zinc-700 ml-2
            transition-colors duration-500 hover:bg-gray-300 dark:hover:bg-zinc-600
            hover:text-gray-800 dark:hover:text-gray-100 dark:text-gray-100"
      onclick={createNewMessage}
    >
      <Icon icon="mage:message-dots-round-plus" />
    </button>
  </div>

  <div class="flex-1 overflow-hidden relative">
    <div class="absolute inset-0 overflow-y-auto">
      <div class="pb-2">
        {#each filteredConversations as conversation (conversation.contact.id)}
          <div
            animate:flip={{ duration: 200, delay: 0 }}
            transition:fade={{ duration: 150 }}
            class="conversation-item flex flex-row items-center p-2 gap-2 cursor-pointer focus:outline-none focus:ring-0
                        hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors duration-300 rounded-md relative "
            class:bg-gray-200={$currentContact?.id ===
              conversation.contact.id}
            class:dark:bg-zinc-700={$currentContact?.id ===
              conversation.contact.id}
            role="button"
            onclick={() => conversationHandleClick(conversation)}
            tabindex="0"
            onkeydown={(e) => {
              if (e.key === "Enter") {
                conversationHandleClick(conversation);
              }
            }}
          >
            <div class="flex justify-center items-center">
              <div class="relative w-8 h-8">
                <Icon
                  icon="mage:user-circle-fill"
                  class="text-gray-400 w-8 h-8"
                />
                {#if conversation.sms_preview.status === SmsStatus.Unread}
                  <span
                    class="absolute top-0 right-0 w-2 h-2 bg-red-500 rounded-full translate-x-1/2 -translate-y-1/2"
                  ></span>
                {/if}
              </div>
            </div>

            <div class="flex-1">
              <p
                id={`conversation-${conversation.contact.id}`}
                class="text-gray-800 dark:text-gray-100 font-bold"
              >
                {conversation.contact.name}
              </p>
              <p class="text-gray-400 text-xs line-clamp-2 min-h-[2.6em]">
                <span
                  class="bg-gray-400 dark:bg-zinc-700 px-1 py-0.5 mr-0.5 rounded-md text-white"
                >
                  {conversation.sms_preview.device}
                </span>
                {conversation.sms_preview.message}
              </p>

              <p class="text-gray-400 text-xs">
                {formatDate(conversation.sms_preview.timestamp)}
              </p>
            </div>            {#if conversation.contact.new === true}
              <button
                class="absolute right-2 top-1/2 -translate-y-1/2 bg-gray-200 dark:bg-zinc-800
                                rounded-full p-1.5 text-gray-400 hover:text-gray-800 dark:hover:text-gray-100
                                transition-colors duration-300 z-10"
                onclick={(e) => {
                  e.stopPropagation();
                  deleteConversationHandleClick(conversation);
                }}
              >
                <Icon icon="mage:multiply-circle" class="w-5 h-5" />
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .overflow-y-auto::-webkit-scrollbar {
    display: none;
  }

  .overflow-y-auto {
    scrollbar-width: none;
  }
</style>
