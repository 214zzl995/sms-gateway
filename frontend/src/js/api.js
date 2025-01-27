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
     * @param {string|null} [device=null] - Optional device name (for filtering specific devices)
     */
    async getSmsPaginated(page = 1, perPage = 10, device = null) {
        const params = {
            page: page,
            per_page: perPage,
            ...(device && { device }) 
        };

        return FetchApi.get('/api/sms', params);
    }

    /**
     * Send an SMS
     * @param {string} modemId - Modem ID
     * @param {string} number - Target phone number
     * @param {string} message - SMS content
     */
    async sendSms(modemId, number, message) {
        const payload = { modem_id: modemId, number, message };
        return FetchApi.post('/api/sms', payload);
    }

    /**
     * Get all modem information
     */
    async getAllModems() {
        return FetchApi.get('/api/device');
    }
}

// Export as a singleton
export const apiClient = new ApiClient();

// ---------------------------
// Usage examples:
// import { apiClient } from './api';

// Check authentication after login
// const isValid = await apiClient.checkAuth();

// Get the first page of SMS (20 items per page)
// const smsData = await apiClient.getSmsPaginated(1, 20);

// Send an SMS
// await apiClient.sendSms('modem-1', '+123456789', 'Hello World');

// Get SMS for a specific device
// const deviceSms = await apiClient.getDeviceSmsPaginated('device-1', 1, 10);

// Get all modems
// const modems = await apiClient.getAllModems();