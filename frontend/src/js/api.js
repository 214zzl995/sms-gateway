// api.js
import FetchApi from './request';

/**
 * Encapsulates all API calls, automatically handles authentication and global errors
 */
class ApiClient {
    /**
     * Check authentication validity
     */
    async checkAuth() {
        try {
            const response = await FetchApi.get('/api/check');
            return response.status === 204;
        } catch (error) {
            if (error.status === 401) {
                localStorage.removeItem('auth');
                window.location.reload();
            }
            return false;
        }
    }    /**
     * Get paginated SMS list
     * @param {number} [page=1] - Page number
     * @param {number} [perPage=10] - Number of items per page
     * @param {number|null} [contactId=null] - Optional contact ID (for filtering specific contacts)
     * @param {AbortSignal} [signal=null] - Optional AbortSignal to cancel the request
     */
    async getSmsPaginated(page = 1, perPage = 10, contactId = null, signal = null) {
        const params = {
            page: page,
            per_page: perPage,
            contact_id: contactId
        };

        return FetchApi.get('/api/sms', params, undefined, { signal });
    }    /**
     * Send an SMS
     * @param {string} simId - Modem ID
     * @param {object} contact - Target phone number
     * @param {string} message - SMS content
     * @param {boolean} new_message - Whether to send a new message
     */
    async sendSms(simId, contact, message, new_message) {
        const payload = { sim_id: simId, contact, message, new: new_message };
        return FetchApi.post('/api/sms', payload)
    }

    /**
     * @param {any} simId
     */
    async getSimInfo(simId) {
        return await FetchApi.get(`/api/sims/${simId}/info`);
    }

    /**
     * Get all SIM dynamic information
     */
    async getAllSimsInfo() {
        return FetchApi.get('/api/sims/info');
    }

    /**
     * @param {any} simId
     */
    async refreshSms(simId) {
        return FetchApi.get(`/api/sims/${simId}/refresh`)
    }

    async getConversation() {
        return FetchApi.get('/api/conversation')
    }

    async markConversationAsReadAndGetLatest(contactId) {
        return FetchApi.post(`/api/conversations/${contactId}/unread`);
    }

    /**
     * Get all SIM cards information
     */
    async getAllSimCards() {
        return FetchApi.get('/api/sim-cards', {}, 'application/json', {});
    }

    /**
     * Update SIM card alias
     * @param {number} simId - SIM card ID
     * @param {string} alias - New alias
     */
    async updateSimCardAlias(simId, alias) {
        const payload = { alias };
        return FetchApi.put(`/api/sim-cards/${simId}/alias`, payload, {}, 'application/json');
    }

    /**
     * Update SIM card phone number
     * @param {number} simId - SIM card ID
     * @param {string} phoneNumber - New phone number
     */
    async updateSimCardPhoneNumber(simId, phoneNumber) {
        const payload = { phone_number: phoneNumber };
        return FetchApi.put(`/api/sim-cards/${simId}/phone`, payload, {}, 'application/json');
    }
}

// Export as a singleton
export const apiClient = new ApiClient();

