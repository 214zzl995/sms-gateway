import { get, writable } from 'svelte/store';
import { apiClient } from '../js/api';
import { getStorageValue, updateStorageValue } from '../js/storage';
import { EventSourcePolyfill } from 'event-source-polyfill';

export const conversations = writable([]);
export const currentConversation = writable(null);
export const conversationLoading = writable(false);
export const sseConnected = writable(false);

let eventSource = null;
let reconnectTimeout = null;
const RECONNECT_DELAY = 5000; // 5 seconds

const getAuthHeader = () => {
    const auth = sessionStorage.getItem("auth");
    if (auth) {
        const { token } = JSON.parse(auth);
        return { 'Authorization': `Basic ${token}` };
    }
    return {};
};

const connectSSE = () => {
    if (eventSource) {
        eventSource.close();
    }

    const authHeader = getAuthHeader();
    if (!authHeader.Authorization) {
        console.error('SSE Error: Authorization token not found.');
        sseConnected.set(false);
        return;
    }

    const eventSourceInitDict = {
        headers: {
            ...authHeader
        },
        heartbeatTimeout: 45000 // ms, to prevent "No activity within 45000 milliseconds"
    };

    eventSource = new EventSourcePolyfill('/api/sms/sse', eventSourceInitDict);

    eventSource.onopen = () => {
        console.log('SSE connection established.');
        sseConnected.set(true);
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
        }
    };

    eventSource.onmessage = (event) => {
        if (event.data === 'keep-alive') {
            console.log('SSE keep-alive received.');
            return;
        }
        try {
            const messageData = JSON.parse(event.data);
            if (messageData.type === 'new_message') {
                console.log('SSE new_message received:', messageData);
                initConversation();
            }
        } catch (error) {
            console.error('SSE error parsing message data:', error, 'Raw data:', event.data);
        }
    };

    eventSource.onerror = (error) => {
        console.error('SSE connection error:', error);
        sseConnected.set(false);
        if (eventSource) {
            eventSource.close();
        }
        
        if (!reconnectTimeout) {
            console.log(`SSE: Attempting to reconnect in ${RECONNECT_DELAY / 1000} seconds...`);
            reconnectTimeout = setTimeout(() => {
                connectSSE();
            }, RECONNECT_DELAY);
        }
    };

    eventSource.addEventListener('keep-alive', (event) => {
        console.log('SSE keep-alive event received');
    }, false);
};

export const initConversation = () => {
    conversationLoading.set(true);
    connectSSE();

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
