import { get, writable } from 'svelte/store';
import { apiClient } from '../js/api';
import { getStorageValue, updateStorageValue } from '../js/storage';

export const conversations = writable([]);
export const currentConversation = writable(null);
export const conversationLoading = writable(false);
export const sseConnected = writable(false);

let eventSource = null;
let reconnectTimeout = null;
const RECONNECT_DELAY = 5000; 

const connectSSE = () => {
    if (eventSource) {
        eventSource.close();
    }

    eventSource = new EventSource('/api/sms/sse');
    
    eventSource.onopen = () => {
        console.log('SSE连接已建立');
        sseConnected.set(true);
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
        }
    };

    eventSource.onmessage = (event) => {
       
    };

    eventSource.onerror = (error) => {
        console.error('SSE连接错误:', error);
        sseConnected.set(false);
        eventSource.close();
        
        if (!reconnectTimeout) {
            reconnectTimeout = setTimeout(() => {
                console.log('尝试重新连接SSE...');
                connectSSE();
            }, RECONNECT_DELAY);
        }
    };
};

connectSSE();

export const initConversation = () => {
    conversationLoading.set(true);

    apiClient.getConversation().then((res) => {
        getStorageValue("currentConversation").then((storageValue) => {
            if (storageValue !== null && storageValue !== undefined && res.data.find((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id === storageValue.id)) {
                currentConversation.set(storageValue);
            } else {
                if (res.data.length > 0) {
                    updateStorageValue("currentConversation", res.data[0].contact);
                    currentConversation.set(res.data[0].contact);
                } else {
                    currentConversation.set({
                        id: -1,
                        name: "新信息",
                    });
                }
            }
        });
        conversations.set(res.data);

        conversationLoading.set(false);
    });
}

export const changeCurrentConversation = (/** @type {any} */ contact) => {
    if (contact.id === get(currentConversation)?.id) {
        return;
    }
    updateStorageValue("currentConversation", contact);

    if (contact.id === -1 && !get(conversations).find((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id === -1)) {
        console.log("add new conversation");
        conversations.update((conversations) => {
            return [{
                contact: {
                    id: -1,
                    name: "新信息",
                },
                sms_preview: {
                    message: "",
                    read: true,
                    timestamp: new Date().toISOString(),
                },
            }, ...conversations,];
        });
    }
    currentConversation.set(contact);
    if (contact.id === -1) {
        scrollToConversation(contact.id);
    }
}

export const newMessageConcatChange = (/** @type {string} */ conactName) => {
    if (conactName === "") {
        conactName = "新信息";
    }

    conversations.update((conversations) => {
        return [{
            contact: {
                id: -1,
                name: conactName,
            },
            sms_preview: {
                message: "",
                read: true,
                timestamp: new Date().toISOString(),
            },
        }, ...conversations.filter((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id !== -1)];
    });
}

export const deleteConversation = (/** @type {number} */ id) => {
    conversations.update((conversations) => {
        return conversations.filter((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id !== id);
    });
}

export const scrollToConversation = (/** @type {number} */ id) => {
    setTimeout(() => {
        const conversationElement = document.getElementById(`conversation-${id}`);
        if (conversationElement) {
            const conversationItem = conversationElement.closest('.conversation-item');
            if (conversationItem) {
                const scrollContainer = document.querySelector('.absolute.inset-0.overflow-y-auto');
                if (scrollContainer) {
                    const containerRect = scrollContainer.getBoundingClientRect();
                    const elementRect = conversationItem.getBoundingClientRect();
                    const relativeTop = elementRect.top - containerRect.top + scrollContainer.scrollTop;

                    scrollContainer.scrollTo({
                        top: relativeTop,
                        behavior: 'smooth'
                    });
                }
            }
        }
    }, 100);
}

export const conactAddFinish = (/** @type {string} */ name) => {
    const conversation = get(conversations).find((/** @type {{ contact: { id: any; name: any; }; }} */ item) => item.contact.name === name && item.contact.id !== -1);
    if (conversation) {
        currentConversation.set(conversation.contact);
        scrollToConversation(conversation.contact.id);
        conversations.update((conversations) => {
            return conversations.filter((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id !== -1);
        });
    }
}
