<script>
  import Icon from "@iconify/svelte";
  import { apiClient } from "../js/api";
  import { formatTimeRange, formatDate } from "../js/dateFormat";
  import {
    currentConversation,
    conversationLoading,
    newMessageConcatChange,
    conactAddFinish,
  } from "../stores/conversation";
  import { fade } from "svelte/transition";
  import { onDestroy } from "svelte";
  import { devices } from "../stores/devices";

  let messages = $state([]);
  let showNewMessage = $state(false);
  let concatInput = $state(null);
  let concatInputText = $state("");
  let isAddingContact = $state(false);

  let showDeviceDialog = $state(false);

  let loading = $state(true);

  let sendMessageLoading = $state(false);

  let page = $state(1);
  let pageSize = $state(9999999);

  let showLoading = $state(true);
  let loadingTimer = null;

  const loadingDuration = 150;

  $effect(() => {
    if (!$conversationLoading) {
      if ($currentConversation && $currentConversation.id === -1) {
        showNewMessage = true;
        concatInput?.focus();
      } else if ($currentConversation && $currentConversation.id !== -1) {
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
    loading = true;
    if ($currentConversation && $currentConversation.id !== -1) {
      apiClient
        .getSmsPaginated(page, pageSize, $currentConversation.id)
        .then((res) => {
          messages = res.data.data;
          loading = false;
        });
    }

    if ($currentConversation && $currentConversation.id === -1) {
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

  const concatInputHandleKeyDown = (/** @type {any} */ event) => {
    if (event.key === "Enter") {
      concatInputHandleClick();
    }
  };

  const concatInputHandleClick = () => {
    isAddingContact = true;
    conactAddFinish(concatInputText);
    concatInputText = "";
    isAddingContact = false;
  };

  const sendButtonHandleClick = () => {
    showDeviceDialog = true;
  };

  /**
   * Enhanced URL regex that better handles URLs with surrounding punctuation
   * and properly excludes Chinese parentheses (（）) from the URL
   */
  const urlRegex =
    /https?:\/\/[a-zA-Z0-9][-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g;

  /**
   * Formats text by converting URLs into HTML anchor tags
   * @param {string} text - The input text containing URLs to format
   * @returns {string} Text with URLs converted to HTML links
   */
  function formatMessageWithLinks(text) {
    if (!text) return "";

    // Create a copy of the text to work with
    let formattedText = text;

    // Replace URLs with anchor tags
    const matches = text.match(urlRegex);

    if (matches) {
      matches.forEach((url) => {
        // Make sure we're getting the URL without Chinese parentheses
        const cleanUrl = url.replace(/[（）]/g, "");

        formattedText = formattedText.replace(
          url,
          `<a href="${cleanUrl}" target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">${cleanUrl}</a>`
        );
      });
    }

    return formattedText;
  }

  function clickOutside(node) {
    const handleClick = (event) => {
      if (!node.contains(event.target)) {
        showDeviceDialog = false;
      }
    };
    document.addEventListener("click", handleClick, true);

    return {
      destroy() {
        document.removeEventListener("click", handleClick, true);
      },
    };
  }

  onDestroy(() => {
    if (loadingTimer) clearTimeout(loadingTimer);
  });
</script>

<div class="flex flex-col h-full relative">
  <header
    class="bg-gray-100/70 dark:bg-zinc-900/70 backdrop-blur-md p-2 h-12 flex items-center text-gray-400
     text-sm transition-colors duration-300 absolute top-0 left-0 right-0 z-10"
    class:text-gray-600={showNewMessage}
  >
    收件人:
    {#if !$conversationLoading}
      {#if showNewMessage}
        <input
          type="text"
          class="rounded-md p-2 bg-transparent focus:outline-none focus:ring-0"
          bind:value={concatInputText}
          bind:this={concatInput}
          onkeydown={concatInputHandleKeyDown}
        />
      {:else if $currentConversation}
        {$currentConversation.name}
      {/if}
    {/if}
  </header>

  <div class="flex-1 overflow-hidden relative">
    {#if showLoading}
      <div
        class="h-full flex justify-center items-center absolute inset-0 z-9"
        transition:fade={{ duration: loadingDuration }}
      >
        <div
          class="inline-block animate-spin rounded-full border-2 border-t-gray-800 border-gray-300 w-5 h-5"
        ></div>
      </div>
    {:else}
      <div
        class="h-full overflow-y-auto flex flex-col-reverse message-container z-9 absolute inset-0"
        transition:fade={{ duration: loadingDuration }}
      >
        <div class="flex flex-col-reverse gap-2 p-2 w-full mb-20 mt-12">
          {#each messages as message, index}
            <div
              class="flex mb-2"
              class:justify-end={message.send}
              class:justify-start={!message.send}
            >
              <div class="relative">
                <div
                  class="relative px-4 py-2 text-sm max-w-[70%] rounded-lg max-w-4/6
                  {message.send
                    ? 'bg-blue-500 text-white before:bg-blue-500'
                    : 'bg-gray-200 dark:bg-zinc-800 before:bg-gray-200 before:dark:bg-zinc-800'}
                  before:content-[''] before:absolute before:w-2 before:h-2 before:rotate-45 before:top-[10px]
                  {message.send ? 'before:-right-1' : 'before:-left-1'}"
                >
                  {@html formatMessageWithLinks(message.message)}
                </div>
              </div>
            </div>
            {@const timeHeader = formatTimeRange(
              message.timestamp,
              index === messages.length - 1
                ? null
                : messages[index - 1]?.timestamp
            )}
            {#if timeHeader || index === messages.length - 1}
              <div class="flex justify-center text-xs text-gray-400 my-1">
                {timeHeader || formatDate(message.timestamp)}
              </div>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <div
    class="h-20 flex items-center justify-center bg-white/70 dark:bg-zinc-900/70 z-10 backdrop-blur-md absolute bottom-0 left-0 right-0"
  >
    <div
      class="flex items-center justify-between rounded-full p-2 w-4/6 bg-gray-100 dark:bg-zinc-800 relative"
    >
      <input
        type="text"
        class="ml-2 mr-2 bg-transparent focus:outline-none focus:ring-0 flex-1"
      />

      {#if showDeviceDialog}
        <div
          transition:fade={{ duration: 150 }}
          use:clickOutside
          class="absolute bottom-14 right-0 bg-white dark:bg-zinc-700 rounded-lg p-2 min-w-32 bg-zinc-100"
        >
          <ul class="list-none m-0 p-0">
            {#each $devices as device}
              <li>
                <button
                  class="py-1 px-2 bg-transparent hover:bg-gray-200 dark:hover:bg-zinc-600 rounded cursor-pointer w-full flex items-center gap-2 text-sm"
                >
                  <Icon icon="mage:memory-card-fill" class="text-gray-400 w-4 h-4" />
                  {device.name}
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <button
        class="rounded-full flex items-center justify-center hover:text-blue-500 transition-colors duration-300 mr-2"
        onclick={sendButtonHandleClick}
      >
        <Icon icon="mage:direction-up-right-2-fill" class="w-6 h-6" />
      </button>
    </div>
  </div>
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
    height: 5rem;
    display: block;
  }
</style>
