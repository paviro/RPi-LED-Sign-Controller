// State management
let playlistItems = [];
let editingIndex = -1;  // -1 means adding new item
let currentPage = 'playlist';  // Keep track of current page
let selectedBorderEffect = 'none';
let gradientColors = [(255, 0, 0), (0, 255, 0), (0, 0, 255)]; // Default gradient colors

// DOM Ready
document.addEventListener('DOMContentLoaded', function() {
    initializeApp();
});

async function initializeApp() {
    // Try to fetch current playlist from server
    try {
        const response = await fetch('/playlist');
        if (response.ok) {
            const data = await response.json();
            playlistItems = data.items || [];
            
            // We allow empty playlists now, so we don't add a default item
        } else {
            console.error("Failed to fetch playlist, using empty playlist");
            playlistItems = [];
        }
    } catch (error) {
        console.error("Error fetching playlist:", error);
        playlistItems = [];
    }
    
    // Now render the playlist and set up the UI
    renderPlaylistItems();
    updateColorPreview();
    updateScrollControlsVisibility();
    
    // Set up event listeners
    document.getElementById('scroll').addEventListener('change', updateScrollControlsVisibility);
    document.getElementById('add-item-btn').addEventListener('click', () => navigateToEditor(-1));
    document.getElementById('save-item-btn').addEventListener('click', saveItem);
    document.getElementById('cancel-btn').addEventListener('click', navigateToPlaylist);
    document.getElementById('back-to-playlist').addEventListener('click', navigateToPlaylist);
}

// Page navigation functions
function navigateToEditor(index) {
    editingIndex = index;
    const editorTitle = document.getElementById('editor-title');
    
    if (index === -1) {
        // Adding new item
        editorTitle.textContent = 'Add New Message';
        resetEditorFields();
    } else {
        // Editing existing item
        editorTitle.textContent = 'Edit Message';
        populateEditorFields(playlistItems[index]);
    }
    
    // Hide playlist page, show editor page
    document.getElementById('playlist-page').classList.add('hidden');
    const editorPage = document.getElementById('editor-page');
    editorPage.classList.remove('hidden');
    editorPage.classList.add('page-transition');
    
    // Focus on the text input
    document.getElementById('text').focus();
    
    // Update window title and URL for better navigation
    document.title = (index === -1) ? 'Add New Message - LED Sign Controller' : 'Edit Message - LED Sign Controller';
    
    // Update current page tracking
    currentPage = 'editor';
}

function navigateToPlaylist() {
    // Hide editor page, show playlist page
    document.getElementById('editor-page').classList.add('hidden');
    const playlistPage = document.getElementById('playlist-page');
    playlistPage.classList.remove('hidden');
    playlistPage.classList.add('page-transition');
    
    // Reset editing state
    editingIndex = -1;
    
    // Update window title
    document.title = 'LED Sign Controller';
    
    // Update current page tracking
    currentPage = 'playlist';
}

// Handle browser back button
window.addEventListener('popstate', function(event) {
    if (currentPage === 'editor') {
        navigateToPlaylist();
    }
});

function updateScrollControlsVisibility() {
    const isScrolling = document.getElementById('scroll').checked;
    
    // Show/hide appropriate controls based on scrolling choice
    document.getElementById('scroll-controls').style.display = isScrolling ? 'block' : 'none';
    document.getElementById('static-controls').style.display = isScrolling ? 'none' : 'block';
}

function resetEditorFields() {
    document.getElementById('text').value = '';
    document.getElementById('scroll').checked = true;
    document.getElementById('speed').value = 60;
    document.getElementById('speed-value').textContent = 60;
    document.getElementById('repeats').value = 1;
    document.getElementById('duration').value = 10;
    document.getElementById('brightness').value = 100;
    document.getElementById('brightness-value').textContent = '100%';
    document.getElementById('red').value = 255;
    document.getElementById('green').value = 255;
    document.getElementById('blue').value = 255;
    
    // Reset border effect
    selectedBorderEffect = 'none';
    updateBorderEffectSelection();
    
    // Reset gradient colors
    gradientColors = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
    updateGradientColorStops();
    
    updateColorPreview();
    updateScrollControlsVisibility();
}

function populateEditorFields(item) {
    document.getElementById('text').value = item.text;
    document.getElementById('scroll').checked = item.scroll;
    document.getElementById('speed').value = item.speed;
    document.getElementById('speed-value').textContent = item.speed;
    document.getElementById('repeats').value = item.repeat_count;
    document.getElementById('duration').value = item.duration;
    document.getElementById('brightness').value = item.brightness;
    document.getElementById('brightness-value').textContent = item.brightness + '%';
    document.getElementById('red').value = item.color[0];
    document.getElementById('green').value = item.color[1];
    document.getElementById('blue').value = item.color[2];
    
    // Set border effect
    if (item.border_effect) {
        if (typeof item.border_effect === 'string') {
            selectedBorderEffect = item.border_effect;
        } else if (item.border_effect.Gradient) {
            selectedBorderEffect = 'gradient';
            gradientColors = item.border_effect.Gradient.colors || [];
            updateGradientColorStops();
        } else {
            // Handle enum variant
            const effectType = Object.keys(item.border_effect)[0];
            selectedBorderEffect = effectType.toLowerCase();
        }
    } else {
        selectedBorderEffect = 'none';
    }
    
    updateBorderEffectSelection();
    updateColorPreview();
    updateScrollControlsVisibility();
}

async function saveItem() {
    // Create border effect object based on selection
    let borderEffect = null;
    
    switch (selectedBorderEffect) {
        case 'none':
            borderEffect = { None: null };
            break;
        case 'rainbow':
            borderEffect = { Rainbow: null };
            break;
        case 'pulse':
            borderEffect = { Pulse: null };
            break;
        case 'sparkle':
            borderEffect = { Sparkle: null };
            break;
        case 'gradient':
            borderEffect = { Gradient: { colors: getGradientColors() } };
            break;
    }
    
    const item = {
        text: document.getElementById('text').value || "New Item",
        scroll: document.getElementById('scroll').checked,
        color: [
            parseInt(document.getElementById('red').value),
            parseInt(document.getElementById('green').value),
            parseInt(document.getElementById('blue').value)
        ],
        speed: parseFloat(document.getElementById('speed').value),
        brightness: parseInt(document.getElementById('brightness').value),
        duration: parseInt(document.getElementById('duration').value),
        repeat_count: parseInt(document.getElementById('repeats').value),
        border_effect: borderEffect
    };
    
    if (editingIndex === -1) {
        // Add new item
        playlistItems.push(item);
    } else {
        // Update existing item
        playlistItems[editingIndex] = item;
    }
    
    renderPlaylistItems();
    navigateToPlaylist();
    
    // Automatically update the display
    await updatePlaylistOnServer();
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
        const editBtn = document.createElement('button');
        editBtn.className = 'item-action edit';
        editBtn.innerHTML = '✎';
        editBtn.title = 'Edit';
        editBtn.onclick = (e) => { 
            e.stopPropagation(); 
            navigateToEditor(index); 
        };
        
        // Move up button
        const upBtn = document.createElement('button');
        upBtn.className = 'item-action up';
        upBtn.innerHTML = '↑';
        upBtn.title = 'Move Up';
        upBtn.onclick = (e) => { 
            e.stopPropagation(); 
            moveItemUp(index); 
        };
        upBtn.disabled = index === 0;
        upBtn.style.opacity = index === 0 ? '0.5' : '1';
        
        // Move down button
        const downBtn = document.createElement('button');
        downBtn.className = 'item-action down';
        downBtn.innerHTML = '↓';
        downBtn.title = 'Move Down';
        downBtn.onclick = (e) => { 
            e.stopPropagation(); 
            moveItemDown(index); 
        };
        downBtn.disabled = index === playlistItems.length - 1;
        downBtn.style.opacity = index === playlistItems.length - 1 ? '0.5' : '1';
        
        // Delete button
        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'item-action delete';
        deleteBtn.innerHTML = '×';
        deleteBtn.title = 'Delete';
        deleteBtn.onclick = (e) => { 
            e.stopPropagation(); 
            removePlaylistItem(index); 
        };
        
        // Add action buttons to actions div
        actionsEl.appendChild(editBtn);
        actionsEl.appendChild(upBtn);
        actionsEl.appendChild(downBtn);
        actionsEl.appendChild(deleteBtn);
        
        // Make the whole item clickable to edit
        itemElement.onclick = () => navigateToEditor(index);
        
        // Add elements to item
        itemElement.appendChild(colorEl);
        itemElement.appendChild(textEl);
        itemElement.appendChild(detailsEl);
        itemElement.appendChild(actionsEl);
        
        container.appendChild(itemElement);
    });
}

async function removePlaylistItem(index) {
    if (confirm('Are you sure you want to remove this message?')) {
        playlistItems.splice(index, 1);
        renderPlaylistItems();
        await updatePlaylistOnServer();
    }
}

async function moveItemUp(index) {
    if (index > 0) {
        [playlistItems[index], playlistItems[index - 1]] = [playlistItems[index - 1], playlistItems[index]];
        renderPlaylistItems();
        await updatePlaylistOnServer();
    }
}

async function moveItemDown(index) {
    if (index < playlistItems.length - 1) {
        [playlistItems[index], playlistItems[index + 1]] = [playlistItems[index + 1], playlistItems[index]];
        renderPlaylistItems();
        await updatePlaylistOnServer();
    }
}

function setColor(r, g, b) {
    document.getElementById('red').value = r;
    document.getElementById('green').value = g;
    document.getElementById('blue').value = b;
    
    document.getElementById('red-value').textContent = r;
    document.getElementById('green-value').textContent = g;
    document.getElementById('blue-value').textContent = b;
    
    updateColorPreview();
}

function updateColorPreview() {
    const r = document.getElementById('red').value;
    const g = document.getElementById('green').value;
    const b = document.getElementById('blue').value;
    
    document.getElementById('red-value').textContent = r;
    document.getElementById('green-value').textContent = g;
    document.getElementById('blue-value').textContent = b;
    
    document.getElementById('colorPreview').style.backgroundColor = `rgb(${r},${g},${b})`;
}

async function updatePlaylistOnServer() {
    // We allow empty playlists now, so we remove this check
    // if (playlistItems.length === 0) {
    //     showStatus('Please add at least one message to the playlist', 'error');
    //     return false;
    // }
    
    try {
        const playlist = {
            items: playlistItems,
            active_index: 0,
            repeat: true
        };
        
        const response = await fetch('/playlist', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(playlist),
        });
        
        if (response.ok) {
            showStatus('Display updated', 'success');
            return true;
        } else {
            showStatus('Failed to update display', 'error');
            return false;
        }
    } catch (error) {
        showStatus('Error: ' + error.message, 'error');
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

function selectBorderEffect(effect) {
    selectedBorderEffect = effect;
    updateBorderEffectSelection();
    
    // Show/hide gradient color options
    const gradientOptions = document.getElementById('gradient-colors');
    if (gradientOptions) {
        gradientOptions.classList.toggle('visible', effect === 'gradient');
    }
}

function updateBorderEffectSelection() {
    // Update visual selection
    const options = document.querySelectorAll('.effect-option');
    options.forEach(option => {
        option.classList.toggle('selected', option.dataset.effect === selectedBorderEffect);
    });
}

function addGradientColorStop() {
    // Add a new color to the gradient
    gradientColors.push([
        parseInt(document.getElementById('red').value),
        parseInt(document.getElementById('green').value),
        parseInt(document.getElementById('blue').value)
    ]);
    updateGradientColorStops();
}

function updateGradientColorStops() {
    const container = document.getElementById('gradient-color-stops');
    container.innerHTML = '';
    
    gradientColors.forEach((color, index) => {
        const [r, g, b] = color;
        const stopEl = document.createElement('div');
        stopEl.className = 'color-stop';
        
        const previewEl = document.createElement('div');
        previewEl.className = 'color-stop-preview';
        previewEl.style.backgroundColor = `rgb(${r},${g},${b})`;
        previewEl.onclick = () => editGradientColor(index);
        
        const labelEl = document.createElement('span');
        labelEl.textContent = `Color ${index + 1}`;
        
        const removeBtn = document.createElement('button');
        removeBtn.textContent = 'Remove';
        removeBtn.className = 'small-button remove';
        removeBtn.onclick = () => removeGradientColor(index);
        
        stopEl.appendChild(previewEl);
        stopEl.appendChild(labelEl);
        stopEl.appendChild(removeBtn);
        container.appendChild(stopEl);
    });
}

function editGradientColor(index) {
    // Set the color picker to the gradient color
    const [r, g, b] = gradientColors[index];
    document.getElementById('red').value = r;
    document.getElementById('green').value = g;
    document.getElementById('blue').value = b;
    updateColorPreview();
    
    // Highlight the color being edited
    const colorStops = document.querySelectorAll('.color-stop');
    colorStops.forEach((stop, i) => {
        stop.classList.toggle('editing', i === index);
    });
    
    // Add a save button
    const saveBtn = document.createElement('button');
    saveBtn.textContent = 'Save Color';
    saveBtn.className = 'primary-button save-gradient-color';
    saveBtn.onclick = () => {
        saveGradientColor(index);
        saveBtn.remove();
    };
    
    const container = document.getElementById('gradient-colors');
    if (container.querySelector('.save-gradient-color')) {
        container.querySelector('.save-gradient-color').remove();
    }
    container.appendChild(saveBtn);
}

function saveGradientColor(index) {
    // Save the current color to the gradient
    gradientColors[index] = [
        parseInt(document.getElementById('red').value),
        parseInt(document.getElementById('green').value),
        parseInt(document.getElementById('blue').value)
    ];
    updateGradientColorStops();
    
    // Remove editing class
    const colorStops = document.querySelectorAll('.color-stop');
    colorStops.forEach(stop => stop.classList.remove('editing'));
}

function removeGradientColor(index) {
    // Remove a color from the gradient
    if (gradientColors.length > 1) {
        gradientColors.splice(index, 1);
        updateGradientColorStops();
    }
}

function getGradientColors() {
    return gradientColors;
} 