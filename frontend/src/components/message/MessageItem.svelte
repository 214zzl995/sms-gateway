<script>
  import Icon from "@iconify/svelte";
  import { SmsStatus } from "../../stores/conversation";
  import { quintOut } from "svelte/easing";

  let { message, isNewMessage = false } = $props();

  const urlRegex =
    /https?:\/\/[a-zA-Z0-9][-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g;

  const verificationCodeRegex =
    /(?:验证码|校验码|(?<!号)码|code|Code)[^0-9]*([0-9]{4,8})/gi;

  function formatMessage(text) {
    if (!text) return "";

    let formattedText = text;

    const urlMatches = text.match(urlRegex);
    if (urlMatches) {
      urlMatches.forEach((url) => {
        const cleanUrl = url.replace(/[（）]/g, "");
        formattedText = formattedText.replace(
          url,
          `<a href="${cleanUrl}" target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">${cleanUrl}</a>`,
        );
      });
    }

    const codeMatches = text.matchAll(verificationCodeRegex);
    for (const match of codeMatches) {
      const [fullMatch, code] = match;
      const prefix = fullMatch.slice(0, fullMatch.lastIndexOf(code));
      formattedText = formattedText.replace(
        fullMatch,
        `${prefix}<span class="inline-flex items-center justify-center gap-1 bg-gray-100/80 dark:bg-zinc-700/80 hover:bg-gray-200 dark:hover:bg-zinc-600 text-blue-600 dark:text-blue-400 transition-colors duration-200 cursor-pointer px-1.5 py-0.5 rounded mx-0.5" onclick="(function(event){
            event.preventDefault();
            event.stopPropagation();
            navigator.clipboard.writeText('${code}');
            const toast = document.createElement('div');
            toast.className = 'fixed bottom-24 left-1/2 transform -translate-x-1/2 bg-gray-800 text-white px-4 py-2 rounded-full text-sm opacity-0 transition-opacity duration-300';
            toast.textContent = '验证码已复制';
            document.body.appendChild(toast);
            requestAnimationFrame(() => toast.style.opacity = '1');
            setTimeout(() => {
              toast.style.opacity = '0';
              setTimeout(() => toast.remove(), 300);
            }, 2000);          })(event)" title="点击复制验证码">${code}<svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/></svg></span>`,
      );
    }

    return formattedText;
  }

  function slideDown(node, { duration = 300, easing = quintOut }) {
    if (!isNewMessage) return {};

    const height = node.offsetHeight;

    return {
      duration,
      css: (t) => {
        const eased = easing(t);
        return `
          overflow: hidden;
          height: ${eased * height}px;
          opacity: ${t < 0.5 ? t * 2 : 1};
          transform: translateY(${(1 - eased) * 10}px);
        `;
      },
    };
  }
</script>

<div
  class="flex mb-2 message-wrapper flex-row"
  class:justify-end={message.send}
  class:justify-start={!message.send}
  in:slideDown={{ duration: 300 }}
>
  <div
    class="relative max-w-[70%] md:max-w-[65%] lg:max-w-[60%] xl:max-w-[55%]"
  >
    {#if message.send && message.status !== undefined}
      <div class="absolute top-1/2 -left-6 -translate-y-1/2 transform">
        {#if message.status === SmsStatus.Loading}
          <div
            class="w-3 h-3 border-2 border-gray-300 border-t-gray-600 rounded-full animate-spin"
          ></div>
        {:else if message.status === SmsStatus.Failed}
          <Icon
            icon="mage:information-circle-fill"
            class="text-red-500 w-5 h-5"
          />
        {:else if message.status === SmsStatus.Read}
          <div class="text-green-500 text-xs"></div>
        {/if}
      </div>
    {/if}

    <div
      class="relative px-4 py-2 text-sm rounded-lg
      {message.send
        ? 'bg-gray-700 text-white before:bg-gray-700'
        : 'bg-gray-50 dark:bg-gray-800 before:bg-gray-100 before:dark:bg-gray-800'}
      {message.send ? 'before:-right-1' : 'before:-left-1'}"
    >
      <p class="whitespace-pre-wrap break-words overflow-hidden">
        {@html formatMessage(message.message)}
      </p>
    </div>
  </div>
</div>

<style>
  .message-wrapper {
    will-change: height, transform, opacity;
  }
</style>
