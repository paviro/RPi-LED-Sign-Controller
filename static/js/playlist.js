// Playlist page JS
let playlistItems = [];
let globalBrightness = 100;

// Initialize playlist when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    fetchAndRenderPlaylist();
    setupBrightnessControls();
    updateSliderFill();
});

function setupBrightnessControls() {
    fetchCurrentBrightness();
    
    const brightnessSlider = document.getElementById('global-brightness');
    
    // Update brightness and fill while dragging (with debouncing)
    brightnessSlider.addEventListener('input', function() {
        document.getElementById('brightness-value').textContent = this.value + '%';
        updateSliderFill();
        
        // Debounce the actual update to reduce network requests
        clearTimeout(this.timeout);
        this.timeout = setTimeout(() => {
            updateBrightness();
        }, 50); // Small delay for a responsive feel
    });
}

// Add debounce function to avoid too many requests
function debounce(func, wait) {
    let timeout;
    return function() {
        const context = this;
        const args = arguments;
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(context, args), wait);
    };
}

async function fetchCurrentBrightness() {
    try {
        const response = await fetch('/brightness');
        if (response.ok) {
            const data = await response.json();
            globalBrightness = data.brightness;
            
            // Update the UI
            document.getElementById('global-brightness').value = globalBrightness;
            document.getElementById('brightness-value').textContent = globalBrightness + '%';
            updateSliderFill();
        } else {
            console.error("Failed to fetch brightness");
            // Use default value of 100
            document.getElementById('global-brightness').value = 100;
            document.getElementById('brightness-value').textContent = "100%";
            updateSliderFill();
        }
    } catch (error) {
        console.error("Error fetching brightness:", error);
    }
}

async function updateBrightness() {
    const brightnessInput = document.getElementById('global-brightness');
    const brightness = parseInt(brightnessInput.value);
    
    // Validate brightness value
    if (isNaN(brightness) || brightness < 0 || brightness > 100) {
        // Only show an error message if there's an actual error
        utils.showStatus('Brightness must be between 0 and 100', 'error');
        brightnessInput.value = globalBrightness; // Reset to current value
        document.getElementById('brightness-value').textContent = globalBrightness + '%';
        return;
    }
    
    try {
        await fetch('/brightness', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ brightness }),
        });
        
        // Success - don't show any message
        globalBrightness = brightness;
    } catch (error) {
        // Only show a message for errors
        utils.showStatus('Error: ' + error.message, 'error');
    }
}

async function fetchAndRenderPlaylist() {
    try {
        const response = await fetch('/playlist');
        if (response.ok) {
            const data = await response.json();
            playlistItems = data.items || [];
            renderPlaylistItems();
        } else {
            console.error("Failed to fetch playlist");
            showStatus('Failed to load playlist data', 'error');
            playlistItems = [];
            renderPlaylistItems();
        }
    } catch (error) {
        console.error("Error fetching playlist:", error);
        showStatus('Error: ' + error.message, 'error');
        playlistItems = [];
        renderPlaylistItems();
    }
}

function renderPlaylistItems() {
    const container = document.getElementById('playlist-items');
    container.innerHTML = '';
    
    if (playlistItems.length === 0) {
        container.innerHTML = `
            <div class="empty-playlist-message">
                <p>Your playlist is empty.</p>
                <p>The LED panel will display "Adjust playlist on the web" until you add messages.</p>
                <p>Click the "Add New Message" button below to get started.</p>
            </div>
        `;
        return;
    }
    
    playlistItems.forEach((item, index) => {
        const itemElement = document.createElement('div');
        itemElement.className = 'playlist-item';
        
        // Color indicator
        const colorEl = document.createElement('div');
        colorEl.className = 'item-color';
        colorEl.style.backgroundColor = `rgb(${item.color[0]},${item.color[1]},${item.color[2]})`;
        
        // Text content
        const textEl = document.createElement('div');
        textEl.className = 'item-text';
        textEl.textContent = item.text;
        
        // Details
        const detailsEl = document.createElement('div');
        detailsEl.className = 'item-details';
        if (item.scroll) {
            detailsEl.textContent = `Scrolling · ${item.speed} px/s · ${item.repeat_count} repeats`;
        } else {
            detailsEl.textContent = `Static · ${item.duration} seconds`;
        }
        
        // Action buttons
        const actionsEl = document.createElement('div');
        actionsEl.className = 'item-actions';
        
        // Edit button
        const editBtn = document.createElement('a');
        editBtn.href = `/editor?index=${index}`;
        editBtn.className = 'item-action edit';
        editBtn.innerHTML = '✎';
        editBtn.title = 'Edit';
        
        // Move up button
        const upBtn = document.createElement('button');
        upBtn.className = 'item-action up';
        upBtn.innerHTML = '↑';
        upBtn.title = 'Move Up';
        upBtn.onclick = () => moveItemUp(index);
        upBtn.disabled = index === 0;
        upBtn.style.opacity = index === 0 ? '0.5' : '1';
        
        // Move down button
        const downBtn = document.createElement('button');
        downBtn.className = 'item-action down';
        downBtn.innerHTML = '↓';
        downBtn.title = 'Move Down';
        downBtn.onclick = () => moveItemDown(index);
        downBtn.disabled = index === playlistItems.length - 1;
        downBtn.style.opacity = index === playlistItems.length - 1 ? '0.5' : '1';
        
        // Delete button
        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'item-action delete';
        deleteBtn.innerHTML = '×';
        deleteBtn.title = 'Delete';
        deleteBtn.onclick = () => removePlaylistItem(index);
        
        // Add action buttons to actions div
        actionsEl.appendChild(editBtn);
        actionsEl.appendChild(upBtn);
        actionsEl.appendChild(downBtn);
        actionsEl.appendChild(deleteBtn);
        
        // Add elements to item
        itemElement.appendChild(colorEl);
        itemElement.appendChild(textEl);
        itemElement.appendChild(detailsEl);
        itemElement.appendChild(actionsEl);
        
        // Make the whole item clickable to edit
        itemElement.onclick = (e) => {
            // Only navigate if clicking the item itself, not a button
            if (e.target === itemElement || e.target === textEl || e.target === detailsEl || e.target === colorEl) {
                window.location.href = `/editor?index=${index}`;
            }
        };
        
        container.appendChild(itemElement);
    });
}

async function removePlaylistItem(index) {
    if (confirm('Are you sure you want to remove this message?')) {
        playlistItems.splice(index, 1);
        await updatePlaylistOnServer();
        renderPlaylistItems();
    }
}

async function moveItemUp(index) {
    if (index > 0) {
        [playlistItems[index], playlistItems[index - 1]] = [playlistItems[index - 1], playlistItems[index]];
        await updatePlaylistOnServer();
        renderPlaylistItems();
    }
}

async function moveItemDown(index) {
    if (index < playlistItems.length - 1) {
        [playlistItems[index], playlistItems[index + 1]] = [playlistItems[index + 1], playlistItems[index]];
        await updatePlaylistOnServer();
        renderPlaylistItems();
    }
}

async function updatePlaylistOnServer() {
    try {
        const playlist = {
            items: playlistItems,
            active_index: 0,
            repeat: true,
            brightness: globalBrightness
        };
        
        const response = await fetch('/playlist', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(playlist),
        });
        
        if (response.ok) {
            // Don't show "Display updated" message
            return true;
        } else {
            // Still show error messages
            utils.showStatus('Failed to update display', 'error');
            return false;
        }
    } catch (error) {
        utils.showStatus('Error: ' + error.message, 'error');
        return false;
    }
}

function showStatus(message, type) {
    const statusElement = document.getElementById('status');
    statusElement.textContent = message;
    statusElement.className = type + ' visible';
    
    // Hide the status message after 3 seconds
    setTimeout(() => {
        statusElement.className = statusElement.className.replace('visible', '');
    }, 3000);
}

function updateSliderFill() {
    const slider = document.getElementById('global-brightness');
    const percentage = (slider.value / slider.max) * 100;
    slider.style.backgroundImage = `linear-gradient(to right, var(--primary-color) 0%, var(--primary-color) ${percentage}%, #f5f5f5 ${percentage}%, #f5f5f5 100%)`;
} 