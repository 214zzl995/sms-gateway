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
    }

    /**
     * Get paginated SMS list
     * @param {number} [page=1] - Page number
     * @param {number} [perPage=10] - Number of items per page
     * @param {number|null} [contactId=null] - Optional contact ID (for filtering specific contacts)
     */
    async getSmsPaginated(page = 1, perPage = 10, contactId = null) {
        const params = {
            page: page,
            per_page: perPage,
            contact_id: contactId 
        };

        return FetchApi.get('/api/sms', params);
    }

    /**
     * Send an SMS
     * @param {string} modemId - Modem ID
     * @param {object} contact - Target phone number
     * @param {string} message - SMS content
     */
    async sendSms(modemId, contact, message) {
        const payload = { modem_id: modemId, contact, message };
        return FetchApi.post('/api/sms', payload)
    }

    /**
     * @param {any} modemId
     */
    async getModemsInfo(modemId) {
        return FetchApi.get(`/api/device/${modemId}`)
    }

    /**
     * Get all modem information
     */
    async getAllModems() {
        return FetchApi.get('/api/device');
    }

    /**
     * @param {any} modemId
     */
    async refreshSms(modemId){
        return FetchApi.get(`/api/refresh/${modemId}`)
    }

    async getConversation(){
        return FetchApi.get('/api/conversation')
    }
}

// Export as a singleton
export const apiClient = new ApiClient();

