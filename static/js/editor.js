// Editor page JS
let editingIndex = -1;  // -1 means adding new item
let playlistItems = [];
let colorPicker; // Color picker instance
let gradientColors = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
let selectedBorderEffect = 'none';
let textSegments = []; // To store colored text segments

// Initialize editor when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    initializeColorPicker();
    initializeEditor();
});

function initializeColorPicker() {
    // Initialize Alwan color picker
    colorPicker = new Alwan('#colorPreview', {
        theme: 'light',
        toggle: false,     // Always visible
        popover: false,    // Embed in page
        opacity: true,     // Support alpha channel
        preset: true,      // Use preset button
        color: '#FFFFFF',  // Initial white color
        default: '#FFFFFF',// Default white color
        format: 'rgb',     // Start with RGB format
        copy: true,        // Allow copying to clipboard
        singleInput: true, // Use a single input for color value
        inputs: {
            rgb: true,
            hsl: false,
            hex: true
        }
    });

    // Update the RGB value display when color changes
    colorPicker.on('color', function(event) {
        document.getElementById('color-value').textContent = `RGB: ${event.r}, ${event.g}, ${event.b}`;
    });
}

async function initializeEditor() {
    // Get the index from URL query parameter 
    const urlParams = new URLSearchParams(window.location.search);
    editingIndex = parseInt(urlParams.get('index') || '-1');
    
    // Fetch the current playlist
    try {
        const response = await fetch('/playlist');
        if (response.ok) {
            const data = await response.json();
            playlistItems = data.items || [];
            
            // If editing an existing item, populate the form
            if (editingIndex >= 0 && editingIndex < playlistItems.length) {
                populateEditorFields(playlistItems[editingIndex]);
                document.getElementById('editor-title').textContent = 'Edit Message';
            } else {
                // Adding a new item
                resetEditorFields();
                document.getElementById('editor-title').textContent = 'Add New Message';
                editingIndex = -1;
            }
        } else {
            console.error("Failed to fetch playlist");
            showStatus('Failed to load playlist data', 'error');
            resetEditorFields();
        }
    } catch (error) {
        console.error("Error fetching playlist:", error);
        showStatus('Error: ' + error.message, 'error');
        resetEditorFields();
    }
    
    // Set up event listeners
    document.getElementById('scroll').addEventListener('change', updateScrollControlsVisibility);
    document.getElementById('save-item-btn').addEventListener('click', saveItem);
    
    // Initialize UI components
    updateScrollControlsVisibility();
    
    // Set up rich text editor
    initializeRichTextEditor();
}

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
    
    // Reset color picker to white
    colorPicker.setColor('#FFFFFF');
    document.getElementById('color-value').textContent = 'RGB: 255, 255, 255';
    
    // Reset border effect
    selectedBorderEffect = 'none';
    updateBorderEffectSelection();
    
    // Reset gradient colors
    gradientColors = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
    updateGradientColorStops();
    
    updateScrollControlsVisibility();
}

function populateEditorFields(item) {
    document.getElementById('text').value = item.text;
    document.getElementById('scroll').checked = item.scroll;
    document.getElementById('speed').value = item.speed;
    document.getElementById('speed-value').textContent = item.speed;
    document.getElementById('repeats').value = item.repeat_count;
    document.getElementById('duration').value = item.duration;
    
    // Set color picker to the saved color
    const [r, g, b] = item.color;
    colorPicker.setColor(`rgb(${r}, ${g}, ${b})`);
    document.getElementById('color-value').textContent = `RGB: ${r}, ${g}, ${b}`;
    
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
    
    // Update border effect selection and show/hide gradient UI if needed
    updateBorderEffectSelection();
    const gradientOptions = document.getElementById('gradient-colors');
    if (gradientOptions) {
        gradientOptions.classList.toggle('visible', selectedBorderEffect === 'gradient');
    }
    
    updateScrollControlsVisibility();
}

async function saveItem() {
    // Form validation
    const text = document.getElementById('text').value;
    if (!text.trim()) {
        utils.showStatus('Please enter some text for the message', 'error');
        document.getElementById('text').focus();
        return;
    }
    
    // Speed validation
    const speed = parseFloat(document.getElementById('speed').value);
    if (document.getElementById('scroll').checked && (isNaN(speed) || speed < 10 || speed > 150)) {
        utils.showStatus('Please enter a valid scroll speed between 10 and 150', 'error');
        document.getElementById('speed').focus();
        return;
    }
    
    // Repeats validation
    const repeats = parseInt(document.getElementById('repeats').value);
    if (document.getElementById('scroll').checked && (isNaN(repeats) || repeats < 1)) {
        utils.showStatus('Please enter a valid number of repeats (minimum 1)', 'error');
        document.getElementById('repeats').focus();
        return;
    }
    
    // Duration validation for static text
    const duration = parseInt(document.getElementById('duration').value);
    if (!document.getElementById('scroll').checked && (isNaN(duration) || duration < 1)) {
        utils.showStatus('Please enter a valid duration (minimum 1 second)', 'error');
        document.getElementById('duration').focus();
        return;
    }
    
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
    
    // Get current color from the color picker
    const colorData = colorPicker.getColor();
    const r = Math.round(colorData.r);
    const g = Math.round(colorData.g);
    const b = Math.round(colorData.b);
    
    const item = {
        text: document.getElementById('text').value || "New Item",
        scroll: document.getElementById('scroll').checked,
        color: [r, g, b],
        speed: parseFloat(document.getElementById('speed').value),
        duration: parseInt(document.getElementById('duration').value),
        repeat_count: parseInt(document.getElementById('repeats').value),
        border_effect: borderEffect,
        colored_segments: textSegments.length > 0 ? textSegments : null
    };
    
    // Fetch the current playlist again to make sure we have the latest version
    try {
        const playlistResponse = await fetch('/playlist');
        if (playlistResponse.ok) {
            const data = await playlistResponse.json();
            playlistItems = data.items || [];
        }
    } catch (error) {
        console.error("Error fetching playlist before save:", error);
    }
    
    if (editingIndex === -1) {
        // Add new item
        playlistItems.push(item);
    } else if (editingIndex >= 0 && editingIndex < playlistItems.length) {
        // Update existing item
        playlistItems[editingIndex] = item;
    }
    
    // Update the playlist on the server
    try {
        const playlist = {
            items: playlistItems,
            active_index: 0,
            repeat: true,
            brightness: await getCurrentBrightness()
        };
        
        const response = await fetch('/playlist', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(playlist),
        });
        
        if (response.ok) {
            // Redirect immediately without showing a status message
            window.location.href = '/';
        } else {
            // Still show error messages
            utils.showStatus('Failed to update display', 'error');
        }
    } catch (error) {
        utils.showStatus('Error: ' + error.message, 'error');
    }
}

// Helper function to get current brightness
async function getCurrentBrightness() {
    try {
        const response = await fetch('/brightness');
        if (response.ok) {
            const data = await response.json();
            return data.brightness;
        }
    } catch (error) {
        console.error("Error fetching brightness:", error);
    }
    return 100; // Default to 100 if we can't fetch it
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

// Add these new functions to handle gradient colors
function addGradientColorStop() {
    // Get current color from the color picker
    const colorData = colorPicker.getColor();
    const r = Math.round(colorData.r);
    const g = Math.round(colorData.g);
    const b = Math.round(colorData.b);
    
    // Add a new color to the gradient
    gradientColors.push([r, g, b]);
    updateGradientColorStops();
}

function updateGradientColorStops() {
    const container = document.getElementById('gradient-color-stops');
    if (!container) return;
    
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
    colorPicker.setColor(`rgb(${r}, ${g}, ${b})`);
    
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
    // Save the current color from the color picker to the gradient
    const colorData = colorPicker.getColor();
    gradientColors[index] = [
        Math.round(colorData.r),
        Math.round(colorData.g),
        Math.round(colorData.b)
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
    console.log("Selected effect:", selectedBorderEffect); // Debug
    
    let foundMatch = false;
    options.forEach(option => {
        const isSelected = option.dataset.effect === selectedBorderEffect;
        option.classList.toggle('selected', isSelected);
        if (isSelected) {
            foundMatch = true;
            console.log("Matched with", option.dataset.effect); // Debug
        }
    });
    
    if (!foundMatch) {
        console.warn("No matching effect found for:", selectedBorderEffect); // Debug
    }
}

function initializeRichTextEditor() {
    const richTextEditor = document.getElementById('richTextEditor');
    const applyColorBtn = document.getElementById('applyColorBtn');
    const resetColorBtn = document.getElementById('resetColorBtn');
    
    // Initialize with regular text if editing an existing item
    if (editingIndex >= 0 && editingIndex < playlistItems.length) {
        const item = playlistItems[editingIndex];
        richTextEditor.innerText = item.text;
        
        // If there are colored segments, apply them to the editor
        if (item.colored_segments && item.colored_segments.length > 0) {
            applyColoredSegmentsToEditor(item.text, item.colored_segments);
            textSegments = [...item.colored_segments];
        }
    }
    
    // Apply current color to selected text
    applyColorBtn.addEventListener('click', function() {
        const selection = window.getSelection();
        if (selection.rangeCount > 0) {
            const range = selection.getRangeAt(0);
            
            // Only apply if text is selected within our editor
            if (richTextEditor.contains(range.commonAncestorContainer)) {
                applyColorToSelection(range);
            }
        }
    });
    
    // Reset selection color to default
    resetColorBtn.addEventListener('click', function() {
        const selection = window.getSelection();
        if (selection.rangeCount > 0) {
            const range = selection.getRangeAt(0);
            
            // Only apply if text is selected within our editor
            if (richTextEditor.contains(range.commonAncestorContainer)) {
                resetSelectionColor(range);
            }
        }
    });
    
    // Update hidden text field when rich editor content changes
    richTextEditor.addEventListener('input', updateHiddenTextField);
    
    // Initial value for hidden field
    updateHiddenTextField();
}

function applyColorToSelection(range) {
    // Get current color from the color picker
    const colorData = colorPicker.getColor();
    const r = Math.round(colorData.r);
    const g = Math.round(colorData.g);
    const b = Math.round(colorData.b);
    const colorCSS = `rgb(${r},${g},${b})`;
    
    // Create a span with the color
    const span = document.createElement('span');
    span.className = 'colored-text';
    span.style.color = colorCSS;
    span.setAttribute('data-color', `${r},${g},${b}`);
    
    // Apply the span to the selection
    range.surroundContents(span);
    
    // Update text segments
    updateTextSegments();
}

function resetSelectionColor(range) {
    // Extract the text content
    const selectedText = range.toString();
    
    // Replace the range with plain text
    range.deleteContents();
    range.insertNode(document.createTextNode(selectedText));
    
    // Update text segments
    updateTextSegments();
}

function updateHiddenTextField() {
    const richTextEditor = document.getElementById('richTextEditor');
    const hiddenField = document.getElementById('text');
    
    // Just store the plain text version in the form field
    hiddenField.value = richTextEditor.innerText;
}

function updateTextSegments() {
    const richTextEditor = document.getElementById('richTextEditor');
    textSegments = [];
    
    // Get all colored spans
    const coloredSpans = richTextEditor.querySelectorAll('.colored-text');
    
    // Calculate the plain text
    const plainText = richTextEditor.innerText;
    
    // Process each colored span
    coloredSpans.forEach(span => {
        const spanText = span.innerText;
        const colorAttr = span.getAttribute('data-color');
        
        if (colorAttr) {
            const [r, g, b] = colorAttr.split(',').map(Number);
            
            // Find where this text appears in the plain text
            // This is simplified and may need improvement for complex cases
            const startIndex = plainText.indexOf(spanText);
            if (startIndex !== -1) {
                textSegments.push({
                    start: startIndex,
                    end: startIndex + spanText.length,
                    color: [r, g, b]
                });
            }
        }
    });
}

function applyColoredSegmentsToEditor(text, segments) {
    const richTextEditor = document.getElementById('richTextEditor');
    richTextEditor.innerHTML = ''; // Clear editor
    
    let lastEnd = 0;
    
    // Sort segments by start position
    const sortedSegments = [...segments].sort((a, b) => a.start - b.start);
    
    // Apply each segment
    sortedSegments.forEach(segment => {
        // Text before this segment
        if (segment.start > lastEnd) {
            const beforeText = text.substring(lastEnd, segment.start);
            richTextEditor.appendChild(document.createTextNode(beforeText));
        }
        
        // The colored segment
        const segmentText = text.substring(segment.start, segment.end);
        const span = document.createElement('span');
        span.className = 'colored-text';
        span.style.color = `rgb(${segment.color[0]},${segment.color[1]},${segment.color[2]})`;
        span.setAttribute('data-color', segment.color.join(','));
        span.innerText = segmentText;
        richTextEditor.appendChild(span);
        
        lastEnd = segment.end;
    });
    
    // Text after the last segment
    if (lastEnd < text.length) {
        const afterText = text.substring(lastEnd);
        richTextEditor.appendChild(document.createTextNode(afterText));
    }
} 