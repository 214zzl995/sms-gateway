<script>
    import { apiClient } from "../js/api";
    import Icon from "@iconify/svelte";
    import { formatDate } from "../js/dateFormat";
    import { conversations, currentConversation, changeCurrentConversation } from "../stores/conversation";

    let searchTemporaryIsActive = $state(false);

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
        changeCurrentConversation(conversation.contact.id);
    }


</script>

<div class="w-full">
    <div class="flex justify-between items-center">
        <div
            class="flex items-center border-0 rounded-md px-2 bg-gray-200 dark:bg-gray-700 transition-colors duration-500"
            class:bg-gray-300={searchTemporaryIsActive}
            class:dark:bg-gray-600={searchTemporaryIsActive}
        >
            <Icon icon="mage:search" class="text-gray-400 text-xl" />
            <input
                type="text"
                onfocus={searchHandleFocus}
                class="w-full ml-2 p-1 rounded-md border-0 focus:outline-none focus:ring-0 bg-transparent"
                placeholder="Search"
            />
        </div>

        <button
            class="text-stone-800 text-xl p-2 rounded-full bg-gray-200 dark:bg-gray-700 ml-2 
            transition-colors duration-500 hover:bg-gray-300 dark:hover:bg-gray-600 
            hover:text-gray-800 dark:hover:text-gray-100 dark:text-gray-100"
        >
            <Icon icon="mage:message-dots-round-plus" />
        </button>
    </div>

    <div class="flex flex-col gap-1 overflow-y-auto max-h-[calc(100vh-145px)]">
        {#each $conversations as conversation}
            <div
                class="flex flex-row items-center p-2 gap-2 cursor-pointer focus:outline-none focus:ring-0 
                hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors duration-500 rounded-md"
                class:bg-gray-200={$currentConversation === conversation.contact.id}
                class:dark:bg-gray-700={$currentConversation === conversation.contact.id}
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
                        {#if !conversation.sms_preview.read}
                            <span
                                class="absolute top-0 right-0 w-2 h-2 bg-red-500 rounded-full translate-x-1/2 -translate-y-1/2"
                            ></span>
                        {/if}
                    </div>
                </div>

                <div class="flex-1">
                   
                    <p class="text-gray-800 dark:text-gray-100 font-bold">
                        {conversation.contact.name} 
                    </p>
                    <p class="text-gray-400 text-xs line-clamp-2">
                        <span class="bg-gray-400 dark:bg-gray-700 px-1 py-0.5 mr-0.5 rounded-md text-white">
                            {conversation.sms_preview.device}
                        </span>
                        {conversation.sms_preview.message}
                    </p>
                    
                    <p class="text-gray-400 text-xs">
                        {formatDate(conversation.sms_preview.timestamp)}
                    </p>
                </div>
            </div>
        {/each}
    </div>
</div>
