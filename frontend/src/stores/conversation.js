import { writable } from 'svelte/store';
import { apiClient } from '../js/api';

export const conversations = writable([]);
export const currentConversation = writable(null);

export const initConversation = () => {
    apiClient.getConversation().then((res) => {
        conversations.set(res.data);
    });
}

export const changeCurrentConversation = (id) => {
    currentConversation.set(id);
}


