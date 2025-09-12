<script>
  import { apiClient } from "../../js/api";
  import { formatTimeRange, formatDate } from "../../js/dateFormat";
  import {
    currentContact,
    conversationLoading,
    newMessageConcatChange,
    conactAddFinish,
    markConversationAsRead,
    SmsStatus,
    updateConversationLastMessage,
  } from "../../stores/conversation";
  import { fade } from "svelte/transition";
  import { onDestroy, onMount } from "svelte";
  import MessageHeader from "./MessageHeader.svelte";
  import MessageItem from "./MessageItem.svelte";
  import MessageInput from "./MessageInputOptimized.svelte";
  import LoadingSpinner from "../ui/LoadingSpinner.svelte";

  let messages = $state([]);
  let showNewMessage = $state(false);
  let concatInputText = $state("");
  let isAddingContact = $state(false);
  let loading = $state(true);
  let sendMessageContent = $state("");
  let page = $state(1);
  let pageSize = $state(9999999);
  let showLoading = $state(true);
  let loadingTimer = null;
  let prevConversationId = null;
  let messageContainer = $state(null);
  let isNewMessage = $state(false);
  const loadingDuration = 150;

  $effect(() => {
    if (!$conversationLoading) {
      if ($currentContact && $currentContact.new === true) {
        showNewMessage = true;
      } else if ($currentContact && !$currentContact.new) {
        showNewMessage = false;
      }
    }
  });

  $effect(() => {
    if (!isAddingContact && showNewMessage) {
      newMessageConcatChange(concatInputText);
    }
  });

  $effect(() => {
    if (!$currentContact) return;

    if ($currentContact.id === prevConversationId) return;

    prevConversationId = $currentContact.id;
    loading = true;

    if (!$currentContact.new) {
      apiClient
        .getSmsPaginated(page, pageSize, $currentContact.id)
        .then((res) => {
          isNewMessage = false;
          messages = res.data.data;
          loading = false;
          if (page === 1) {
            markConversationAsRead($currentContact.id);
          }
        });
    } else {
      isNewMessage = false;
      messages = [];
      loading = false;
    }
  });

  $effect(() => {
    if (loading) {
      showLoading = true;
      if (loadingTimer) clearTimeout(loadingTimer);
    } else {
      if (loadingTimer) clearTimeout(loadingTimer);
      loadingTimer = setTimeout(() => {
        showLoading = false;
      }, loadingDuration);
    }
  });

  function handleAddContact() {
    isAddingContact = true;
    conactAddFinish(concatInputText);
    concatInputText = "";
    isAddingContact = false;
  }

  function handleConcatInputTextChange(newText) {
    concatInputText = newText;
  }

  function smoothScrollToBottom() {
    if (messageContainer) {
      messageContainer.scrollTo({
        top: 0,
        behavior: "smooth",
      });
    }
  }

  function handleSendMessage(simId) {
    if (sendMessageContent.trim() === "") {
      return;
    }

    // Mark as new message (to enable animations)
    isNewMessage = true;

    // Add new message
    const newMessage = {
      id: -1,
      message: sendMessageContent,
      send: true,
      timestamp: new Date(),
      status: SmsStatus.Loading,
    };

    // Add message to array
    messages = [newMessage, ...messages];

    // Clear input
    sendMessageContent = "";

    // Start smooth scroll animation
    setTimeout(() => {
      smoothScrollToBottom();
    }, 300);

    const concat =
      $currentContact.new === true
        ? {
            id: $currentContact.id,
            name: concatInputText,
          }
        : $currentContact;

    apiClient
      .sendSms(simId, concat, newMessage.message, $currentContact.new ?? false)
      .then((res) => {
        isNewMessage = false;
        const messageId = res.data;
        messages = messages.map((msg) => {
          if (msg.id === -1 && msg.message === newMessage.message) {
            return { ...msg, status: SmsStatus.Read, id: messageId.sms_id };
          }
          return msg;
        });
      })
      .catch((err) => {
        isNewMessage = false;
        messages = messages.map((msg) => {
          if (msg.id === -1 && msg.message === newMessage.message) {
            return { ...msg, status: SmsStatus.Failed };
          }
          return msg;
        });
        console.error("发送消息失败:", err);
      })
      .finally(() => {
        updateConversationLastMessage(
          $currentContact.id,
          newMessage.message,
          simId,
          concatInputText || $currentContact.name
        );
        showNewMessage = false;
        concatInputText = "";
      });
  };

  // Add auto-update functionality
  function handleMessageUpdate(event) {
    const { messages: newMessages, silentUpdate } = event.detail;

    if (!newMessages || newMessages.length === 0) return;

    // Disable animation effects
    if (silentUpdate) {
      isNewMessage = false;
    }

    // Remove duplicates to avoid repeated messages
    const existingIds = new Set(messages.map((msg) => msg.id));
    const uniqueNewMessages = newMessages.filter(
      (msg) => !existingIds.has(msg.id)
    );

    if (uniqueNewMessages.length > 0) {
      messages = [...uniqueNewMessages, ...messages];
    }
  }

  onMount(() => {
    window.addEventListener("update-messages", handleMessageUpdate);
  });

  onDestroy(() => {
    if (loadingTimer) clearTimeout(loadingTimer);
    window.removeEventListener("update-messages", handleMessageUpdate);
  });

  onDestroy(() => {
    if (loadingTimer) clearTimeout(loadingTimer);
  });
</script>

<div class="flex flex-col h-full relative">
  <MessageHeader
    {showNewMessage}
    bind:concatInputText
    onConcatInputTextChange={handleConcatInputTextChange}
    onAddContact={handleAddContact}
  />

  <div class="flex-1 overflow-hidden relative">
    <LoadingSpinner show={showLoading} duration={loadingDuration} />
    {#if !showLoading}
      <div
        class="h-full overflow-y-auto flex flex-col-reverse message-container z-9 absolute inset-0"
        bind:this={messageContainer}
        transition:fade={{ duration: loadingDuration }}
      >
        <div class="flex flex-col-reverse gap-2 p-2 w-full mb-20 mt-12">
          {#each messages as message, index (message.id)}
            <MessageItem {message} {isNewMessage} />
            {@const timeHeader = formatTimeRange(
              message.timestamp,
              index === messages.length - 1
                ? null
                : messages[index - 1]?.timestamp
            )}
            {#if timeHeader || index === messages.length - 1}
              <div
                class="flex justify-center text-xs text-gray-400 my-1"
                in:fade={{ duration: 300, delay: 100 }}
              >
                {timeHeader || formatDate(message.timestamp)}
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <MessageInput
    bind:sendMessageContent
    {showNewMessage}
    {concatInputText}
    onSend={handleSendMessage}
  />

  
</div>

<style>
  .message-container::-webkit-scrollbar {
    width: 6px;
  }

  .message-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .message-container::-webkit-scrollbar-thumb {
    background: #888;
    border-radius: 3px;
  }

  .message-container::-webkit-scrollbar-button:start:decrement {
    height: 3rem;
    display: block;
  }

  .message-container::-webkit-scrollbar-button:end:increment {
    height: 10rem;
    display: block;
  }
</style>
