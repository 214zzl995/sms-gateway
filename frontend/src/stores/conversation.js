import { get, writable } from 'svelte/store';
import { apiClient } from '../js/api';
import { getStorageValue, updateStorageValue } from '../js/storage';
import { EventSourcePolyfill } from 'event-source-polyfill';
import { generateUUID } from '../js/uuid';

export const conversations = writable([]);
export const currentContact = writable(null);
export const conversationLoading = writable(false);
export const sseConnected = writable(false);

export const SmsStatus = {
    Unread: 0,
    Read: 1,
    Loading: 2,
    Failed: 3,
};

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
        sseConnected.set(true);
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
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

    eventSource.addEventListener('conversations', (event) => {
        let data = event.data;
        const newConversations = JSON.parse(data);
        const currentConvId = get(currentContact)?.id;

        const updatedCurrentConv = newConversations.find(conv => conv.contact.id === currentConvId);

        if (updatedCurrentConv && currentConvId !== -1) {

            apiClient.markConversationAsReadAndGetLatest(currentConvId).then(res => {
                if (res && res.data && res.data.length > 0) {
                    window.dispatchEvent(new CustomEvent("update-messages", {
                        detail: {
                            messages: res.data,
                            silentUpdate: true
                        }
                    }));
                }
            });
        }

        conversations.update((currentConversations) => {
            const conversationMap = new Map();
            currentConversations.forEach(conv => {
                conversationMap.set(conv.contact.id, conv);
            });

            newConversations.forEach(newConv => {
                if (newConv.contact.id === currentConvId) {
                    newConv.sms_preview.status = SmsStatus.Read;
                }
                conversationMap.delete(newConv.contact.id);
            });

            newConversations.sort((a, b) => {
                const dateA = new Date(a.sms_preview.timestamp);
                const dateB = new Date(b.sms_preview.timestamp);
                return dateB.getTime() - dateA.getTime();
            });

            return [...newConversations, ...Array.from(conversationMap.values())];
        });
    });

};

export const initConversation = () => {
    conversationLoading.set(true);
    connectSSE();

    const pendingUpdate = sessionStorage.getItem('pendingContactUpdate');
    if (pendingUpdate) {
        try {
            const { contactId, contactName } = JSON.parse(pendingUpdate);
            if (contactId && contactName) {
                sessionStorage.removeItem('pendingContactUpdate');
            }
        } catch (e) {
            console.error('Failed to parse pending contact update:', e);
            sessionStorage.removeItem('pendingContactUpdate');
        }
    }

    apiClient.getConversation().then((res) => {
        getStorageValue("currentConversation").then((storageValue) => {
            if (storageValue !== null && storageValue !== undefined && res.data.find((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id === storageValue.id)) {
                currentContact.set(storageValue);
            } else {                if (res.data.length > 0) {
                    updateStorageValue("currentConversation", res.data[0].contact);
                    currentContact.set(res.data[0].contact);                } else {
                    // 使用前端生成的 UUID，避免立即创建联系人
                    const uuid = generateUUID();
                    const newContact = {
                        id: uuid,
                        name: "新信息",
                        new: true,
                    };
                    currentContact.set(newContact);
                    updateStorageValue("currentConversation", newContact);
                    
                    // 添加到对话列表
                    conversations.update(currentConversations => {
                        return [{
                            contact: newContact,
                            sms_preview: {
                                message: "",
                                status: SmsStatus.Read,
                                timestamp: new Date().toISOString(),
                                sim_id: ""
                            }
                        }, ...currentConversations];
                    });
                }
            }
        });
        conversations.set(res.data);

        conversationLoading.set(false);
    });
}

export const changeCurrentConversation = (/** @type {any} */ contact) => {
    if (contact.id === get(currentContact)?.id) {
        return;
    }
    updateStorageValue("currentConversation", contact);

    if (contact.new === true && !get(conversations).find((/** @type {{ contact: { new: any; }; }} */ item) => item.contact.new === true)) {
        conversations.update((conversations) => {
            return [{
                contact: {
                    id: contact.id,
                    name: "新信息",
                    new: true,
                },
                sms_preview: {
                    message: "",
                    status: SmsStatus.Read,
                    timestamp: new Date().toISOString(),
                },
            }, ...conversations,];
        });
    }
    currentContact.set(contact);
    if (contact.new === true) {
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
                id: get(currentContact)?.id || 0,
                name: conactName,
                new: true,
            },
            sms_preview: {
                message: "",
                status: SmsStatus.Read,
                timestamp: new Date().toISOString(),
            },
        }, ...conversations.filter((/** @type {{ contact: { new: any; }; }} */ item) => !item.contact.new)];
    });
}

export const deleteConversation = (/** @type {string} */ id) => {
    conversations.update((conversations) => {
        return conversations.filter((/** @type {{ contact: { id: any; }; }} */ item) => item.contact.id !== id);
    });
}

export const scrollToConversation = (/** @type {string} */ id) => {
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
    const conversation = get(conversations).find((/** @type {{ contact: { id: any; name: any; new: any; }; }} */ item) => item.contact.name === name && !item.contact.new);
    if (conversation) {
        currentContact.set(conversation.contact);
        scrollToConversation(conversation.contact.id);
        conversations.update((conversations) => {
            return conversations.filter((/** @type {{ contact: { new: any; }; }} */ item) => !item.contact.new);
        });
    }
}

export const markConversationAsRead = (/** @type {string} */ contactId) => {
    if (contactId === undefined || contactId === null) {
        return;
    }

    conversations.update(currentConversations => {
        return currentConversations.map(conv => {
            if (conv.contact.id === contactId && conv.sms_preview.status === SmsStatus.Unread) {
                return {
                    ...conv,
                    sms_preview: {
                        ...conv.sms_preview,
                        status: SmsStatus.Read
                    }
                };
            }
            return conv;
        });
    });
};

/**
 * 更新会话的最后一条消息信息
 * @param {string} contactId - 联系人ID
 * @param {string} message - 最新消息内容
 * @param {string} simId - SIM卡ID
 * @param {string} contactName - 联系人名称
 * @returns {void}
 */
export const updateConversationLastMessage = (
    /** @type {string} */ contactId,
    /** @type {string} */ message,
    /** @type {string} */ simId,
    /** @type {string} */ contactName,
) => {
    if (contactId === undefined || contactId === null) {
        return;
    }

    if (get(currentContact)?.id === contactId) {
        currentContact.set({
            ...get(currentContact),
            name: contactName,
            new: false
        });
    }

    conversations.update(currentConversations => {
        return currentConversations.map(conv => {
            if (conv.contact.id === contactId) {
                return {
                    ...conv,
                    contact: {
                        ...conv.contact,
                        name: contactName,
                        new: false
                    },
                    sms_preview: {
                        ...conv.sms_preview,
                        sim_id: simId,
                        message: message,
                        timestamp: new Date().toISOString(),
                        status: SmsStatus.Read
                    }
                };
            }
            return conv;
        });
        
    });
};

export const createNewContactName = () => {
    const timestamp = new Date().getTime();
    const randomPart = Math.random().toString(36).substring(2, 8);
    const randomName = `新信息 ${timestamp}-${randomPart}`;
    return randomName;
}


