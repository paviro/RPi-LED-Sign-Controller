// Utility functions used across multiple pages
const utils = {
    // Show status message function - we'll still use this for errors
    showStatus: function(message, type) {
        // Only show error messages, skip success messages to make UI snappier
        if (type !== 'success') {
            const statusElement = document.getElementById('status');
            statusElement.textContent = message;
            statusElement.className = `status visible ${type}`;
            
            // Auto-hide after 5 seconds
            setTimeout(() => {
                statusElement.classList.remove('visible');
            }, 5000);
        }
    },
    
    // Format color for display
    formatColor: function(color) {
        if (Array.isArray(color)) {
            return `RGB: ${color[0]}, ${color[1]}, ${color[2]}`;
        }
        return '';
    },
    
    // Fetch API wrapper with error handling - modified to not show success messages
    fetchWithErrorHandling: async function(url, options = {}) {
        try {
            const response = await fetch(url, options);
            if (!response.ok) {
                throw new Error(`HTTP error ${response.status}`);
            }
            return await response.json();
        } catch (error) {
            this.showStatus(`Network error: ${error.message}`, 'error');
            throw error;
        }
    }
}; 