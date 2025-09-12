import { writable } from 'svelte/store';
import { apiClient } from '../js/api.js';

// SIM cards store
export const simCards = writable([]);

// Loading state
export const simCardsLoading = writable(false);

// Functions to manage SIM cards
export const simCardActions = {
    // Load all SIM cards
    async loadAll() {
        simCardsLoading.set(true);
        try {
            const response = await apiClient.getAllSimCards();
            simCards.set(response.data);
        } catch (error) {
            console.error('Failed to load SIM cards:', error);
            simCards.set([]);
        } finally {
            simCardsLoading.set(false);
        }
    },

    // Update SIM card alias
    async updateAlias(simId, alias) {
        try {
            await apiClient.updateSimCardAlias(simId, alias);
            // Refresh the list after update
            await this.loadAll();
            return true;
        } catch (error) {
            console.error('Failed to update SIM card alias:', error);
            return false;
        }
    },

    // Update SIM card phone number
    async updatePhoneNumber(simId, phoneNumber) {
        try {
            await apiClient.updateSimCardPhoneNumber(simId, phoneNumber);
            // Refresh the list after update
            await this.loadAll();
            return true;
        } catch (error) {
            console.error('Failed to update SIM card phone number:', error);
            return false;
        }
    }
};