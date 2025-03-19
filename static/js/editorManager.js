/**
 * Editor Manager - Using TextEditor Component
 */

// Speed preset configuration settings (px/sec)
const SPEED_PRESETS = {
    slow: 30,
    normal: 50,
    fast: 150
};

document.addEventListener('alpine:init', () => {
    Alpine.data('editorApp', () => ({
        editingIndex: -1,
        playlistItems: [],
        gradientColors: [],
        selectedBorderEffect: 'none',
        statusMessage: '',
        statusType: '',
        statusVisible: false,
        textColorHex: '#ffffff',
        textEditor: null,
        textSegments: [],
        speedPreset: 'normal', // Default preset
        formData: {
            text: '',
            scroll: true,
            speed: SPEED_PRESETS.normal, // Use preset config value as default
            repeat_count: 1,
            duration: 10,
            color: [255, 255, 255]
        },
        
        init() {
            this.initializeEditor();
        },
        
        async initializeEditor() {
            // Get the index from URL query parameter 
            const urlParams = new URLSearchParams(window.location.search);
            this.editingIndex = parseInt(urlParams.get('index') || '-1');
            
            // Fetch the current playlist
            try {
                const response = await fetch('/playlist');
                if (response.ok) {
                    const data = await response.json();
                    this.playlistItems = data.items || [];
                    
                    // If editing an existing item, populate the form
                    if (this.editingIndex >= 0 && this.editingIndex < this.playlistItems.length) {
                        this.populateEditorFields(this.playlistItems[this.editingIndex]);
                    } else {
                        this.editingIndex = -1;
                    }
                } else {
                    this.showStatus('Failed to load playlist data', 'error');
                }
            } catch (error) {
                this.showStatus('Error: ' + error.message, 'error');
            }
            
            // Set initial text color
            this.textColorHex = this.rgbToHex(
                this.formData.color[0], 
                this.formData.color[1], 
                this.formData.color[2]
            );
            
            // Initialize the TextEditor component
            this.initializeTextEditor();
        },
        
        // Initialize the TextEditor component
        initializeTextEditor() {
            // Create the TextEditor instance with options
            this.textEditor = new TextEditor({
                editorId: 'richTextEditor',
                defaultColor: this.textColorHex,
                onChange: (text, segments) => {
                    // Update form data when text changes
                    this.formData.text = text;
                    this.textSegments = segments;
                }
            });
            
            // Get the initial content and set it in the editor
            const initialText = this.formData.text;
            const initialSegments = (this.editingIndex >= 0) ? 
                this.playlistItems[this.editingIndex].colored_segments : null;
                
            // Set the initial content in the editor
            if (initialText) {
                this.textEditor.setContent(initialText, initialSegments);
            }
            
            // Setup event handlers for color picker and rainbow buttons
            this.setupEditorEventHandlers();
        },
        
        // Setup event handlers for editor-related UI elements
        setupEditorEventHandlers() {
            // Color picker change event
            const colorInput = document.querySelector('input[type="color"]');
            if (colorInput) {
                colorInput.addEventListener('input', (e) => {
                    // Update our model
                    this.textColorHex = e.target.value;
                    
                    // Convert hex to RGB
                    const rgb = this.hexToRgb(e.target.value);
                    this.formData.color = rgb;
                });
            }
            
            // Nothing else needed here as the Alpine directives handle the rainbow buttons
        },
        
        // Handle text color change when input value changes
        handleTextColorChange(hexColor) {
            // Update our model
            this.textColorHex = hexColor;
            
            // Convert hex to RGB
            const rgb = this.hexToRgb(hexColor);
            this.formData.color = rgb;
            
            // Pass to the text editor
            if (this.textEditor) {
                this.textEditor.handleTextColorChange(hexColor);
            }
        },
        
        // Rainbow words 
        randomizeAlternateWords() {
            if (this.textEditor) {
                const success = this.textEditor.randomizeAlternateWords();
                if (success) {
                    this.showStatus('Rainbow words applied!', 'success');
                } else {
                    this.showStatus('Please enter some text first', 'error');
                }
            }
        },
        
        // Rainbow letters
        randomizeLetters() {
            if (this.textEditor) {
                const success = this.textEditor.randomizeLetters();
                if (success) {
                    this.showStatus('Rainbow letters applied!', 'success');
                } else {
                    this.showStatus('Please enter some text first', 'error');
                }
            }
        },
        
        // Update hidden field - ensure we get the text from the editor component
        updateHiddenTextField() {
            if (this.textEditor) {
                this.formData.text = this.textEditor.getContent();
                this.textSegments = this.textEditor.textSegments;
            }
        },
        
        // Convert RGB to Hex for color inputs
        rgbToHex(r, g, b) {
            return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
        },
        
        // Convert Hex to RGB
        hexToRgb(hex) {
            // Remove # if present
            hex = hex.replace('#', '');
            
            // Convert 3-digit hex to 6-digits
            if (hex.length === 3) {
                hex = hex.split('').map(c => c + c).join('');
            }
            
            // Parse the hex values
            const r = parseInt(hex.substring(0, 2), 16);
            const g = parseInt(hex.substring(2, 4), 16);
            const b = parseInt(hex.substring(4, 6), 16);
            
            return [r, g, b];
        },
        
        // Updated method to use the speed preset configuration
        setSpeedPreset(preset) {
            this.speedPreset = preset;
            
            // Set speed based on preset using configuration
            switch (preset) {
                case 'slow':
                    this.formData.speed = SPEED_PRESETS.slow;
                    break;
                case 'normal':
                    this.formData.speed = SPEED_PRESETS.normal;
                    break;
                case 'fast':
                    this.formData.speed = SPEED_PRESETS.fast;
                    break;
                case 'custom':
                    // Keep the current value or set a default
                    if (!this.formData.speed || this.formData.speed < 10) {
                        this.formData.speed = SPEED_PRESETS.normal;
                    }
                    break;
            }
        },
        
        // Modified function to use the speed preset configuration
        populateEditorFields(item) {
            // Set form fields
            this.formData.text = item.text;
            this.formData.scroll = item.scroll;
            this.formData.speed = item.speed;
            this.formData.repeat_count = item.repeat_count;
            this.formData.duration = item.duration;
            this.formData.color = item.color;
            
            // Determine speed preset based on the loaded speed value
            if (item.speed <= SPEED_PRESETS.slow + 5) {
                this.speedPreset = 'slow';
            } else if (item.speed <= SPEED_PRESETS.normal + 15) {
                this.speedPreset = 'normal';
            } else if (item.speed <= SPEED_PRESETS.fast + 5) {
                this.speedPreset = 'fast';
            } else {
                this.speedPreset = 'custom';
            }
            
            // Update text color hex
            this.textColorHex = this.rgbToHex(...item.color);
            
            // Reset border effect to default
            this.selectedBorderEffect = 'none';
            this.gradientColors = [];
            
            // Handle border effect
            if (item.border_effect) {
                console.log("Border effect data:", JSON.stringify(item.border_effect));
                
                // Check if it's a string (legacy format)
                if (typeof item.border_effect === 'string') {
                    this.selectedBorderEffect = item.border_effect.toLowerCase();
                } 
                // Check if it's an object (newer format)
                else if (typeof item.border_effect === 'object') {
                    // Get the effect type from the object keys
                    const effectKeys = Object.keys(item.border_effect);
                    
                    if (effectKeys.length > 0) {
                        const effectType = effectKeys[0]; // Get the first (and should be only) key
                        
                        // Convert to lowercase for our frontend representation
                        this.selectedBorderEffect = effectType.toLowerCase();
                        
                        // Special handling for gradient, pulse, and sparkle colors
                        if ((effectType === 'Gradient' || effectType === 'Pulse' || effectType === 'Sparkle') && 
                            item.border_effect[effectType] && 
                            item.border_effect[effectType].colors) {
                            
                            this.gradientColors = item.border_effect[effectType].colors.length > 0 ? 
                                [...item.border_effect[effectType].colors] : [];
                        }
                    }
                }
            }
            
            console.log("Selected border effect:", this.selectedBorderEffect);
        },
        
        selectBorderEffect(effect) {
            this.selectedBorderEffect = effect;
            
            // If selecting gradient, pulse, or sparkle and no colors exist, initialize with an empty array
            if ((effect === 'gradient' || effect === 'pulse' || effect === 'sparkle') && this.gradientColors.length === 0) {
                this.gradientColors = [];
            }
        },
        
        addGradientColor() {
            // Add a new color to the gradient using the current text color
            const rgb = this.hexToRgb(this.textColorHex);
            this.gradientColors.push(rgb);
        },
        
        updateGradientColor(index, hexColor) {
            // Convert hex to RGB and update the gradient color
            this.gradientColors[index] = this.hexToRgb(hexColor);
        },
        
        removeGradientColor(index) {
            if (this.gradientColors.length > 1) {
                this.gradientColors.splice(index, 1);
            } else if (this.gradientColors.length === 1) {
                // If this is the last color, just clear the array
                this.gradientColors = [];
            }
        },
        
        showStatus(message, type) {
            this.statusMessage = message;
            this.statusType = type;
            this.statusVisible = true;
            
            setTimeout(() => {
                this.statusVisible = false;
            }, 5000);
        },
        
        async saveItem() {
            // Form validation
            if (!this.formData.text.trim()) {
                this.showStatus('Please enter some text for the message', 'error');
                document.getElementById('richTextEditor').focus();
                return;
            }
            
            // Create border effect object based on selection
            let borderEffect = null;
            
            switch (this.selectedBorderEffect) {
                case 'none':
                    borderEffect = { None: null };
                    break;
                case 'rainbow':
                    borderEffect = { Rainbow: null };
                    break;
                case 'pulse':
                    // Use the same color management as gradient
                    if (this.gradientColors.length > 0) {
                        borderEffect = { Pulse: { colors: this.gradientColors } };
                    } else {
                        // If no colors, use an empty array which will default to text color
                        borderEffect = { Pulse: { colors: [] } };
                    }
                    break;
                case 'sparkle':
                    // Use the same color management as gradient
                    if (this.gradientColors.length > 0) {
                        borderEffect = { Sparkle: { colors: this.gradientColors } };
                    } else {
                        // If no colors, use an empty array which will default to text color
                        borderEffect = { Sparkle: { colors: [] } };
                    }
                    break;
                case 'gradient':
                    // Only save gradient if there are colors
                    if (this.gradientColors.length > 0) {
                        borderEffect = { Gradient: { colors: this.gradientColors } };
                    } else {
                        // Default to None if no gradient colors are set
                        borderEffect = { None: null };
                        this.selectedBorderEffect = 'none';
                    }
                    break;
            }
            
            // Get segments from the text editor
            const textSegments = this.textEditor ? this.textEditor.textSegments : this.textSegments;
            
            const item = {
                text: this.formData.text,
                scroll: this.formData.scroll,
                color: this.formData.color,
                speed: parseFloat(this.formData.speed),
                duration: parseInt(this.formData.duration),
                repeat_count: parseInt(this.formData.repeat_count),
                border_effect: borderEffect,
                colored_segments: textSegments.length > 0 ? textSegments : null
            };
            
            // Fetch the current playlist again to make sure we have the latest version
            try {
                const playlistResponse = await fetch('/playlist');
                if (playlistResponse.ok) {
                    const data = await playlistResponse.json();
                    this.playlistItems = data.items || [];
                }
            } catch (error) {
                console.error("Error fetching playlist before save:", error);
            }
            
            if (this.editingIndex === -1) {
                // Add new item
                this.playlistItems.push(item);
            } else if (this.editingIndex >= 0 && this.editingIndex < this.playlistItems.length) {
                // Update existing item
                this.playlistItems[this.editingIndex] = item;
            }
            
            // Get current brightness
            let brightness = 100;
            try {
                const response = await fetch('/brightness');
                if (response.ok) {
                    const data = await response.json();
                    brightness = data.brightness;
                }
            } catch (error) {
                console.error("Error fetching brightness:", error);
            }
            
            // Update the playlist on the server
            try {
                const playlist = {
                    items: this.playlistItems,
                    active_index: 0,
                    repeat: true,
                    brightness: brightness
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
                    this.showStatus('Failed to update display', 'error');
                }
            } catch (error) {
                this.showStatus('Error: ' + error.message, 'error');
            }
        },
        
        // Increment repeats count
        incrementRepeats() {
            this.formData.repeat_count = parseInt(this.formData.repeat_count) + 1;
        },
        
        // Decrement repeats count (minimum 1)
        decrementRepeats() {
            if (this.formData.repeat_count > 1) {
                this.formData.repeat_count = parseInt(this.formData.repeat_count) - 1;
            }
        },
        
        // Increment duration
        incrementDuration() {
            this.formData.duration = parseInt(this.formData.duration) + 1;
        },
        
        // Decrement duration (minimum 1)
        decrementDuration() {
            if (this.formData.duration > 1) {
                this.formData.duration = parseInt(this.formData.duration) - 1;
            }
        }
    }));
});