<script>
  import Icon from "@iconify/svelte";
  import { SmsStatus } from "../../stores/conversation";
  import { quintOut } from "svelte/easing";
  import { fade, fly } from "svelte/transition";

  let {
    message,
    isNewMessage = false
  } = $props();

  let copied = $state(false);

  const urlRegex =
    /https?:\/\/[a-zA-Z0-9][-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g;

  const verificationCodeRegex =
    /(?:验证码|校验码|(?<!号)码|code|Code)[^0-9]*([0-9]{4,8})/gi;

  function formatMessage(text) {
    if (!text) return "";

    let formattedText = text;

    // Format URLs
    const urlMatches = text.match(urlRegex);
    if (urlMatches) {
      urlMatches.forEach((url) => {
        const cleanUrl = url.replace(/[（）]/g, "");
        formattedText = formattedText.replace(
          url,
          `<a href="${cleanUrl}" target="_blank" rel="noopener noreferrer" class="underline decoration-1 underline-offset-2 hover:decoration-2 transition-all">${cleanUrl}</a>`
        );
      });
    }

    // Format verification codes
    const codeMatches = text.matchAll(verificationCodeRegex);
    for (const match of codeMatches) {
      const [fullMatch, code] = match;
      const prefix = fullMatch.slice(0, fullMatch.lastIndexOf(code));
      formattedText = formattedText.replace(
        fullMatch,
        `${prefix}<span class="verification-code" data-code="${code}">${code}</span>`
      );
    }

    return formattedText;
  }

  function handleCodeClick(event) {
    const codeElement = event.target.closest('.verification-code');
    if (codeElement) {
      const code = codeElement.dataset.code;
      navigator.clipboard.writeText(code);
      
      // Show toast
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    }
  }

  function slideIn(node, { duration = 400, easing = quintOut }) {
    if (!isNewMessage) return {};

    return {
      duration,
      css: (t) => {
        const eased = easing(t);
        return `
          opacity: ${eased};
          transform: translateY(${(1 - eased) * 20}px) scale(${0.95 + eased * 0.05});
        `;
      },
    };
  }
</script>

<div
  class="flex mb-3 message-wrapper {message.send ? 'flex-row-reverse' : 'flex-row'}"
  in:slideIn={{ duration: 400 }}
>
  <div class="relative max-w-[75%] md:max-w-[65%] lg:max-w-[60%] xl:max-w-[55%]">
    <!-- Message bubble -->
    <div
      class="relative group"
      
    >
      <!-- Message content -->
      <div
        class="relative px-4 py-2.5 rounded-2xl shadow-sm
        transition-all duration-200
        {message.send
          ? 'bg-gradient-to-br from-blue-500 to-blue-600 text-white shadow-blue-500/10 hover:shadow-lg hover:shadow-blue-500/20'
          : 'bg-gray-50 dark:bg-zinc-800 text-gray-900 dark:text-gray-100 border border-gray-200 dark:border-zinc-700 hover:bg-gray-100 dark:hover:bg-zinc-750'}"
      >
        <!-- Message text -->
        <div class="text-[15px] leading-relaxed whitespace-pre-wrap break-words">
          {@html formatMessage(message.message)}
        </div>

        <!-- Message tail (speech bubble arrow) -->
        <div
          class="absolute top-3 {message.send ? '-right-1.5' : '-left-1.5'}"
        >
          <svg
            width="10"
            height="16"
            viewBox="0 0 10 16"
            class="{message.send ? 'text-blue-600' : 'text-gray-50 dark:text-zinc-800'} {message.send ? '' : 'rotate-180'}"
          >
            <path
              d="M 0 0 Q 10 0 10 8 Q 10 16 0 16 Q 5 8 0 0"
              fill="currentColor"
              class="{message.send ? '' : 'stroke-gray-200 dark:stroke-zinc-700'}"
              stroke-width="{message.send ? '0' : '1'}"
            />
          </svg>
        </div>
      </div>

      <!-- Message metadata -->
      <div class="flex items-center gap-2 mt-1.5 {message.send ? 'flex-row-reverse' : 'flex-row'}">
        <!-- Time -->
        <span class="text-xs text-gray-400 dark:text-gray-500">
          {new Date(message.timestamp || Date.now()).toLocaleTimeString('en-US', {
            hour: '2-digit',
            minute: '2-digit'
          })}
        </span>

        <!-- Status indicator for sent messages -->
        {#if message.send && message.status !== undefined}
          <div class="flex items-center">
            {#if message.status === SmsStatus.Loading}
              <Icon
                icon="carbon:circle-dash"
                class="w-4 h-4 text-gray-400 animate-spin"
              />
            {:else if message.status === SmsStatus.Failed}
              <div class="flex items-center gap-1 text-red-500">
                <Icon icon="carbon:warning" class="w-4 h-4" />
                <span class="text-xs">Failed</span>
              </div>
            {:else if message.status === SmsStatus.Sent}
              <Icon
                icon="carbon:checkmark"
                class="w-4 h-4 text-gray-400"
              />
            {:else if message.status === SmsStatus.Delivered}
              <div class="flex -space-x-1">
                <Icon icon="carbon:checkmark" class="w-4 h-4 text-gray-400" />
                <Icon icon="carbon:checkmark" class="w-4 h-4 text-gray-400" />
              </div>
            {:else if message.status === SmsStatus.Read}
              <div class="flex -space-x-1">
                <Icon icon="carbon:checkmark" class="w-4 h-4 text-blue-500" />
                <Icon icon="carbon:checkmark" class="w-4 h-4 text-blue-500" />
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<!-- Toast notification -->
{#if copied}
  <div
    class="fixed bottom-24 left-1/2 transform -translate-x-1/2 z-50"
    in:fly={{ y: 20, duration: 200 }}
    out:fade={{ duration: 200 }}
  >
    <div class="flex items-center gap-2 bg-gray-800 dark:bg-zinc-700 text-white px-4 py-2.5 rounded-full shadow-lg">
      <Icon icon="carbon:checkmark-filled" class="w-4 h-4 text-green-400" />
      <span class="text-sm font-medium">验证码已复制</span>
    </div>
  </div>
{/if}

<style lang="postcss">
  .message-wrapper {
    will-change: transform, opacity;
  }

  /* Verification code styling */
  :global(.verification-code) {
    @apply inline-flex items-center gap-1.5 px-2 py-0.5 mx-0.5 rounded-lg;
    @apply font-mono font-semibold tracking-wider;
    @apply cursor-pointer transition-all duration-200;
    position: relative;
  }

  :global(.bg-gradient-to-br .verification-code) {
    @apply bg-white/20 hover:bg-white/30;
    @apply text-white;
  }

  :global(.bg-gray-50 .verification-code),
  :global(.dark .bg-zinc-800 .verification-code) {
    @apply bg-blue-50 dark:bg-blue-950/30;
    @apply text-blue-600 dark:text-blue-400;
    @apply hover:bg-blue-100 dark:hover:bg-blue-950/50;
  }

  :global(.verification-code::after) {
    content: '';
    display: inline-block;
    width: 14px;
    height: 14px;
    margin-left: 4px;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Cpath fill='currentColor' d='M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z'/%3E%3C/svg%3E");
    background-size: contain;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  :global(.verification-code:hover::after) {
    opacity: 1;
  }

  /* URL link styling in sent messages */
  :global(.bg-gradient-to-br a) {
    @apply text-white font-medium;
  }

  /* URL link styling in received messages */
  :global(.bg-gray-50 a),
  :global(.dark .bg-zinc-800 a) {
    @apply text-blue-600 dark:text-blue-400 font-medium;
  }

  /* Smooth animation for status indicators */
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.8);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  :global(.message-wrapper > div > div) {
    animation: fadeIn 0.3s ease-out;
  }
</style>