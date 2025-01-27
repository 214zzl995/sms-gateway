import { writable } from 'svelte/store';
import { apiClient } from '../js/api';

export const devices = writable([]);

export const initDeices = () => {
    apiClient.getAllModems().then(
        (res) => {
            devices.set(res.data)
        }
    )
}
