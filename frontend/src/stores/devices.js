import { writable } from 'svelte/store';
import { apiClient } from '../js/api';

export const devices = writable([]);

export const initDeices = () => {
    getDevices()
}

export const refreshDevices = () => {
    getDevices()
}

const getDevices = () => {
    apiClient.getAllModems().then(
        (res) => {
            devices.set(res.data)
        }
    )
}
