/**
 * Playlist Manager
 * Manages the playlist functionality for the LED sign
 */

// Initialize the Alpine.js component
document.addEventListener('alpine:init', () => {
    Alpine.data('playlistApp', () => ({
        playlistItems: [],
        brightness: 100,
        statusMessage: '',
        statusType: '',
        statusVisible: false,
        
        init() {
            this.fetchAndRenderPlaylist();
            this.fetchCurrentBrightness();
            
            // Add window resize event to update details when screen size changes
            window.addEventListener('resize', this.updateResponsiveElements.bind(this));
        },
        
        async fetchAndRenderPlaylist() {
            try {
                const response = await fetch('/playlist');
                if (response.ok) {
                    const data = await response.json();
                    this.playlistItems = data.items || [];
                } else {
                    this.showStatus('Failed to load playlist data', 'error');
                }
            } catch (error) {
                this.showStatus('Error: ' + error.message, 'error');
            }
        },
        
        async fetchCurrentBrightness() {
            try {
                const response = await fetch('/brightness');
                if (response.ok) {
                    const data = await response.json();
                    this.brightness = data.brightness;
                    this.$nextTick(() => this.updateSliderFill());
                }
            } catch (error) {
                console.error("Error fetching brightness:", error);
            }
        },
        
        debouncedUpdateBrightness: function() {
            clearTimeout(this._timeout);
            this._timeout = setTimeout(() => this.updateBrightness(), 50);
        },
        
        async updateBrightness() {
            try {
                await fetch('/brightness', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ brightness: parseInt(this.brightness) })
                });
            } catch (error) {
                this.showStatus('Error: ' + error.message, 'error');
            }
        },
        
        showStatus(message, type) {
            if (type !== 'success' || true) { // Always show errors, optionally show success
                this.statusMessage = message;
                this.statusType = type;
                this.statusVisible = true;
                
                setTimeout(() => {
                    this.statusVisible = false;
                }, 5000);
            }
        },
        
        updateSliderFill() {
            const slider = document.getElementById('global-brightness');
            const percentage = (slider.value / slider.max) * 100;
            slider.style.backgroundImage = `linear-gradient(to right, var(--primary-color) 0%, var(--primary-color) ${percentage}%, #f5f5f5 ${percentage}%, #f5f5f5 100%)`;
        },
        
        getContentTypeLabel(item) {
            if (!item.content_type) return 'Text';
            
            switch (item.content_type) {
                case 'Text':
                    return 'Text';
                default:
                    return 'Text';
            }
        },
        
        getItemDetails(item) {
            // Use actual window width directly for more reliable detection
            const width = window.innerWidth;
            
            // For medium-small screens
            if (width <= 680 && width > 480) {
                return item.scroll ? "Scrolling" : "Static";
            }
            
            // For larger screens, show full details
            if (item.scroll) {
                // Add non-breaking spaces to prevent awkward wrapping
                return `Scrolling · ${item.speed}\u00A0px/s · ${item.repeat_count}\u00A0repeats`;
            } else {
                return `Static · ${item.duration}\u00A0seconds`;
            }
        },
        
        editItem(index, event) {
            if (event.target.closest('.item-actions') === null) {
                window.location.href = `/editor?index=${index}`;
            }
        },
        
        async removePlaylistItem(index) {
            if (confirm('Are you sure you want to remove this message?')) {
                this.playlistItems.splice(index, 1);
                await this.updatePlaylistOnServer();
            }
        },
        
        async moveItemUp(index) {
            if (index > 0) {
                [this.playlistItems[index], this.playlistItems[index - 1]] = 
                [this.playlistItems[index - 1], this.playlistItems[index]];
                await this.updatePlaylistOnServer();
            }
        },
        
        async moveItemDown(index) {
            if (index < this.playlistItems.length - 1) {
                [this.playlistItems[index], this.playlistItems[index + 1]] = 
                [this.playlistItems[index + 1], this.playlistItems[index]];
                await this.updatePlaylistOnServer();
            }
        },
        
        async updatePlaylistOnServer() {
            try {
                const playlist = {
                    items: this.playlistItems,
                    active_index: 0,
                    repeat: true,
                    brightness: parseInt(this.brightness)
                };
                
                const response = await fetch('/playlist', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(playlist)
                });
                
                if (!response.ok) {
                    this.showStatus('Failed to update display', 'error');
                }
            } catch (error) {
                this.showStatus('Error: ' + error.message, 'error');
            }
        },
        
        // Add this new method to handle responsive updates
        updateResponsiveElements() {
            // Force Alpine to re-render the playlist items
            this.playlistItems = [...this.playlistItems];
        }
    }));
}); 