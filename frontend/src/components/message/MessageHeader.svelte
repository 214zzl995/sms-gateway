<script>
  import { currentContact, conversationLoading } from "../../stores/conversation";

  let {
    showNewMessage = false,
    concatInputText = $bindable(""),
    onConcatInputTextChange = () => {},
    onAddContact = () => {}
  } = $props();

  let concatInput = $state(null);

  $effect(() => {
    if (showNewMessage && concatInput) {
      concatInput.focus();
    }
  });

  function handleKeyDown(event) {
    if (event.key === "Enter") {
      onAddContact();
    }
  }

  function handleInput(event) {
    concatInputText = event.target.value;
    onConcatInputTextChange(concatInputText);
  }
</script>

<header
  class="bg-gray-100/70 dark:bg-zinc-900/70 backdrop-blur-md p-2 h-12 flex items-center
   text-sm transition-colors duration-300 absolute top-0 left-0 right-0 z-10"
  class:text-gray-600={showNewMessage}
  class:text-gray-400={!showNewMessage}
  class:dark:text-gray-200={showNewMessage}
  class:dark:text-gray-400={!showNewMessage}
>
  收件人:
  {#if !$conversationLoading}
    {#if showNewMessage}
      <input
        type="text"
        class="rounded-md p-2 bg-transparent focus:outline-none focus:ring-0"
        bind:value={concatInputText}
        bind:this={concatInput}
        onkeydown={handleKeyDown}
        oninput={handleInput}
      />
    {:else if $currentContact}
      {$currentContact.name}
    {/if}
  {/if}
</header>