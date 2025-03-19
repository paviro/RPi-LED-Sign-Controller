/**
 * Text Editor Component
 * Handles rich text editing for LED sign messages
 */

class TextEditor {
    constructor(options = {}) {
        // Default options
        this.options = {
            editorId: 'richTextEditor',
            defaultColor: '#ffffff',
            onChange: null,
            ...options
        };

        // Component state
        this.textSegments = [];
        this.textColorHex = this.options.defaultColor;
        
        // Initialize the editor
        this.init();
    }

    init() {
        this.richTextEditor = document.getElementById(this.options.editorId);
        if (!this.richTextEditor) {
            console.error(`Editor element with ID ${this.options.editorId} not found`);
            return;
        }

        // Ensure the editor is editable
        this.richTextEditor.contentEditable = 'true';
        
        // Set default text color
        const rgb = this.hexToRgb(this.textColorHex);
        this.richTextEditor.style.color = `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
        
        // Set up event listeners
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Update segments on input
        this.richTextEditor.addEventListener('input', () => {
            this.updateTextSegments();
            if (typeof this.options.onChange === 'function') {
                this.options.onChange(this.getContent(), this.textSegments);
            }
        });
        
        // Listen for cursor position changes to update color picker
        this.richTextEditor.addEventListener('mouseup', this.updateColorPickerFromCursor.bind(this));
        this.richTextEditor.addEventListener('keyup', this.updateColorPickerFromCursor.bind(this));
        
        // Add click handler to color input if it exists
        const colorInput = document.querySelector('input[type="color"]');
        if (colorInput) {
            // Add click handler to update to current color when opened
            colorInput.addEventListener('click', () => {
                this.updateColorPickerFromCursor();
            });
            
            // Add change handler to ensure apply on close
            colorInput.addEventListener('change', (e) => {
                this.handleTextColorChange(e.target.value);
            });
        }
    }

    // Get the current content as plain text
    getContent() {
        return this.richTextEditor.textContent || '';
    }

    // Set the content of the editor
    setContent(text, coloredSegments = null) {
        if (!coloredSegments || coloredSegments.length === 0) {
            // Simple text without colored segments
            this.richTextEditor.textContent = text;
        } else {
            // Apply colored segments
            this.applyColoredSegmentsToEditor(text, coloredSegments);
        }
        
        // Update segments
        this.updateTextSegments();
    }

    // Handle text color changes - revised to better handle selections in rainbow text
    handleTextColorChange(hexColor) {
        // Update the color hex value
        this.textColorHex = hexColor;
        
        // Convert hex to RGB
        const rgb = this.hexToRgb(hexColor);
        
        // Get the current selection
        const selection = window.getSelection();
        
        // If there's selected text, apply the color to the selection
        if (selection.rangeCount > 0 && !selection.isCollapsed) {
            const range = selection.getRangeAt(0);
            
            // Only apply if text is selected within our editor
            if (this.richTextEditor.contains(range.commonAncestorContainer)) {
                // Save selection before any operations
                const selectedRange = range.cloneRange();
                const selectedText = selection.toString();
                
                if (selectedText.trim().length > 0) {
                    // Extract range information before modifications
                    const startContainer = selectedRange.startContainer;
                    const startOffset = selectedRange.startOffset;
                    const endContainer = selectedRange.endContainer;
                    const endOffset = selectedRange.endOffset;
                    
                    // Check if we're dealing with rainbow text
                    const hasRainbowSpans = this.richTextEditor.querySelectorAll('.rainbow-word, .rainbow-letter').length > 0;
                    
                    if (hasRainbowSpans) {
                        // This is a more complex case - we need to handle multiple spans
                        this.handleSelectionAcrossRainbowSpans(selectedRange, hexColor, rgb);
                    } else {
                        // Simpler case - just replace the selected text with a colored span
                        const fragment = document.createDocumentFragment();
                        const span = document.createElement('span');
                        span.className = 'colored-text';
                        span.style.color = hexColor;
                        span.setAttribute('data-color', rgb.join(','));
                        span.textContent = selectedText;
                        fragment.appendChild(span);
                        
                        // Delete the selected content and insert our new fragment
                        selectedRange.deleteContents();
                        selectedRange.insertNode(fragment);
                        
                        // Place cursor after the inserted span
                        this.placeCursorAfterNode(span);
                    }
                    
                    // Update text segments
                    this.updateTextSegments();
                }
            }
        } 
        // If there's no selection (just a cursor), try to color the current word/span
        else if (selection.rangeCount > 0) {
            const range = selection.getRangeAt(0);
            
            // Only apply if cursor is within our editor
            if (this.richTextEditor.contains(range.commonAncestorContainer)) {
                // Find the current word/span at cursor position
                const currentSpan = this.getCurrentWordOrSpan(range);
                
                // If we found a span, update its color
                if (currentSpan) {
                    // Update the span's color
                    currentSpan.style.color = hexColor;
                    currentSpan.setAttribute('data-color', rgb.join(','));
                    
                    // Update text segments
                    this.updateTextSegments();
                    
                    // Keep the cursor in the same position
                    this.restoreSelection(range);
                } else {
                    // If no span was found, just update the default color for new text
                    document.execCommand('styleWithCSS', false, true);
                    document.execCommand('foreColor', false, hexColor);
                }
            }
        } else {
            // If no selection, just update the default color for new text
            document.execCommand('styleWithCSS', false, true);
            document.execCommand('foreColor', false, hexColor);
        }
        
        // Update the color picker display - this helps visual feedback
        const colorInput = document.querySelector('input[type="color"]');
        if (colorInput && colorInput.value !== hexColor) {
            colorInput.value = hexColor;
        }
        
        // Call onChange if provided
        if (typeof this.options.onChange === 'function') {
            this.options.onChange(this.getContent(), this.textSegments);
        }
    }

    // New method to handle selections that cross multiple rainbow spans
    handleSelectionAcrossRainbowSpans(range, hexColor, rgb) {
        // Get all spans in the editor
        const spans = this.richTextEditor.querySelectorAll('span');
        const selectedSpans = [];
        
        // Find all spans that are fully or partially within the selection
        spans.forEach(span => {
            if (range.intersectsNode(span)) {
                selectedSpans.push(span);
            }
        });
        
        if (selectedSpans.length > 0) {
            // Get the selected text content
            const selectedText = window.getSelection().toString();
            
            // Create a new range with the first and last selected spans
            const newRange = document.createRange();
            
            // Set the range to encompass all selected spans
            if (selectedSpans.length === 1) {
                // If just one span is selected, we might need to handle partial selection
                const span = selectedSpans[0];
                
                // For a partial selection within a single span
                if (range.startContainer === span.firstChild && 
                    range.endContainer === span.firstChild) {
                    // Extract just the portion of text that was selected
                    const spanText = span.textContent;
                    const selectedPortion = spanText.substring(range.startOffset, range.endOffset);
                    
                    // Create a new span for the selected text
                    const newSpan = document.createElement('span');
                    newSpan.className = 'colored-text';
                    
                    // If the original was a letter span, maintain that for consistency
                    if (span.classList.contains('word-letter') || span.classList.contains('rainbow-letter')) {
                        newSpan.classList.add('word-letter');
                    }
                    
                    newSpan.style.color = hexColor;
                    newSpan.setAttribute('data-color', rgb.join(','));
                    newSpan.textContent = selectedPortion;
                    
                    // Split the text node if needed
                    const textNode = span.firstChild;
                    
                    // If selection starts after beginning of text
                    if (range.startOffset > 0) {
                        const beforeText = spanText.substring(0, range.startOffset);
                        const beforeSpan = span.cloneNode(false);
                        beforeSpan.textContent = beforeText;
                        span.parentNode.insertBefore(beforeSpan, span);
                    }
                    
                    // Insert our new colored span
                    span.parentNode.insertBefore(newSpan, span);
                    
                    // If selection ends before end of text
                    if (range.endOffset < spanText.length) {
                        const afterText = spanText.substring(range.endOffset);
                        const afterSpan = span.cloneNode(false);
                        afterSpan.textContent = afterText;
                        span.parentNode.insertBefore(afterSpan, span);
                    }
                    
                    // Remove the original span
                    span.parentNode.removeChild(span);
                    
                    // Place cursor after the new span
                    this.placeCursorAfterNode(newSpan);
                    return;
                }
            }
            
            // For selections across multiple spans, we'll replace the entire selection
            const fragment = document.createDocumentFragment();
            const newSpan = document.createElement('span');
            
            // Use proper classes based on context
            newSpan.className = 'colored-text';
            
            // If this is a selection of a single letter, add letter class
            if (selectedText.length === 1 && !/\s/.test(selectedText)) {
                newSpan.classList.add('word-letter');
            }
            
            newSpan.style.color = hexColor;
            newSpan.setAttribute('data-color', rgb.join(','));
            newSpan.textContent = selectedText;
            fragment.appendChild(newSpan);
            
            // Delete the original selection
            range.deleteContents();
            
            // Insert the new fragment
            range.insertNode(fragment);
            
            // Place cursor after the inserted span
            this.placeCursorAfterNode(newSpan);
        }
    }

    // Apply rainbow colors to words
    randomizeAlternateWords() {
        const text = this.richTextEditor.textContent || '';
        
        // If no text, do nothing
        if (!text.trim()) {
            return false;
        }
        
        // First, let's add some special CSS to make sure there are no spaces
        // between word spans (same as letters)
        const styleElement = document.createElement('style');
        styleElement.id = 'rainbow-text-styles';
        
        // Remove existing style if present to avoid duplicates
        const existingStyle = document.getElementById('rainbow-text-styles');
        if (existingStyle) {
            existingStyle.remove();
        }
        
        styleElement.textContent = `
            .rainbow-word, .word-letter {
                display: inline;
                white-space: pre;
                margin: 0;
                padding: 0;
                border-spacing: 0;
                font-size: inherit;
                line-height: inherit;
            }
            
            #richTextEditor {
                white-space: pre-wrap;
                word-break: break-word;
            }
        `;
        document.head.appendChild(styleElement);
        
        // Create a sequence of vibrant rainbow colors
        const rainbowColors = [
            [255, 0, 0],      // Red
            [255, 127, 0],    // Orange
            [255, 255, 0],    // Yellow
            [0, 255, 0],      // Green
            [0, 0, 255],      // Blue
            [75, 0, 130],     // Indigo
            [148, 0, 211]     // Violet
        ];
        
        // We'll build the HTML string directly
        let newHtml = '';
        
        // Split into words and non-words (whitespace, punctuation)
        // This regex identifies word boundaries while preserving everything
        const wordRegex = /(\S+)(\s*)/g;
        let match;
        let colorIndex = 0;
        
        // Start at the beginning of the text
        let lastIndex = 0;
        
        // Process each word and its following whitespace
        while ((match = wordRegex.exec(text)) !== null) {
            const [fullMatch, word, whitespace] = match;
            
            // Get a rainbow color for this word
            const rgb = rainbowColors[colorIndex % rainbowColors.length];
            const colorStr = `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`;
            
            // Add the colored word span with both classes for compatibility
            newHtml += `<span class="colored-text rainbow-word" data-word-style="rainbow" style="color:${colorStr}" data-color="${rgb.join(',')}">${word}</span>`;
            
            // Add any whitespace directly (not wrapped in a span)
            newHtml += whitespace;
            
            // Move to next color
            colorIndex++;
            
            // Update lastIndex to track where we are in the text
            lastIndex = match.index + fullMatch.length;
        }
        
        // Add any remaining text
        if (lastIndex < text.length) {
            newHtml += text.substring(lastIndex);
        }
        
        // Set the HTML directly - this is key to preserving exact whitespace
        this.richTextEditor.innerHTML = newHtml;
        
        // Update the text segments
        this.updateTextSegments();
        
        // Focus the editor and place cursor at the end
        this.richTextEditor.focus();
        
        // Move cursor to the end
        const range = document.createRange();
        const selection = window.getSelection();
        range.selectNodeContents(this.richTextEditor);
        range.collapse(false); // false means collapse to end
        selection.removeAllRanges();
        selection.addRange(range);
        
        // Call onChange if provided
        if (typeof this.options.onChange === 'function') {
            this.options.onChange(this.getContent(), this.textSegments);
        }
        
        return true;
    }

    // Apply rainbow colors to letters
    randomizeLetters() {
        const text = this.richTextEditor.textContent || '';
        
        // If no text, do nothing
        if (!text.trim()) {
            return false;
        }
        
        // First, let's add some special CSS to make sure there are no spaces
        // between letter spans
        const styleElement = document.createElement('style');
        styleElement.id = 'rainbow-text-styles';
        
        // Remove existing style if present to avoid duplicates
        const existingStyle = document.getElementById('rainbow-text-styles');
        if (existingStyle) {
            existingStyle.remove();
        }
        
        styleElement.textContent = `
            .word-letter {
                display: inline;
                white-space: pre;
                margin: 0;
                padding: 0;
                border-spacing: 0;
                font-size: inherit;
                line-height: inherit;
            }
            
            .rainbow-word, .word-letter {
                display: inline;
                white-space: pre;
                margin: 0;
                padding: 0;
                border-spacing: 0;
                font-size: inherit;
                line-height: inherit;
            }
            
            #richTextEditor {
                white-space: pre-wrap;
                word-break: break-word;
            }
        `;
        document.head.appendChild(styleElement);
        
        // Create a sequence of vibrant rainbow colors
        const rainbowColors = [
            [255, 0, 0],      // Red
            [255, 70, 0],     // Red-Orange
            [255, 127, 0],    // Orange
            [255, 200, 0],    // Yellow-Orange
            [255, 255, 0],    // Yellow
            [150, 255, 0],    // Yellow-Green
            [0, 255, 0],      // Green
            [0, 255, 150],    // Green-Cyan
            [0, 255, 255],    // Cyan
            [0, 150, 255],    // Cyan-Blue
            [0, 0, 255],      // Blue
            [75, 0, 130],     // Indigo
            [148, 0, 211]     // Violet
        ];
        
        // Count non-whitespace characters to distribute colors properly
        const nonWhitespaceChars = text.replace(/\s+/g, '').length;
        
        // We'll build the HTML string directly for precision
        let newHtml = '';
        
        // Keep track of which color to use (only for non-whitespace characters)
        let colorIndex = 0;
        
        // Process each character
        for (let i = 0; i < text.length; i++) {
            const char = text[i];
            
            // For whitespace characters, preserve them exactly
            if (char === ' ' || char === '\n' || char === '\t') {
                newHtml += char;
                continue;
            }
            
            // Calculate the rainbow color index based on position
            // This creates a smooth rainbow pattern across all letters
            const normalizedIndex = Math.floor((colorIndex / Math.max(1, nonWhitespaceChars - 1)) * (rainbowColors.length - 1));
            const rgb = rainbowColors[normalizedIndex];
            const colorStr = `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`;
            
            // Add the colored character span with both classes for compatibility
            newHtml += `<span class="colored-text word-letter rainbow-letter" data-letter-style="rainbow" style="color:${colorStr}" data-color="${rgb.join(',')}">${char}</span>`;
            
            // Move to next position for non-whitespace characters
            colorIndex++;
        }
        
        // Set the HTML directly - this avoids any browser interpretation issues
        this.richTextEditor.innerHTML = newHtml;
        
        // Update the text segments
        this.updateTextSegments();
        
        // Focus the editor and place cursor at the end
        this.richTextEditor.focus();
        
        // Move cursor to the end
        const range = document.createRange();
        const selection = window.getSelection();
        range.selectNodeContents(this.richTextEditor);
        range.collapse(false); // false means collapse to end
        selection.removeAllRanges();
        selection.addRange(range);
        
        // Call onChange if provided
        if (typeof this.options.onChange === 'function') {
            this.options.onChange(this.getContent(), this.textSegments);
        }
        
        return true;
    }

    // Update the text segments based on the current editor content
    updateTextSegments() {
        this.textSegments = [];
        
        // Get all colored elements
        const coloredElements = this.richTextEditor.querySelectorAll('[data-color]');
        
        if (coloredElements.length === 0) {
            return; // No colored elements to process
        }
        
        // Build a comprehensive map of the text with positions
        const textContent = this.richTextEditor.textContent;
        
        // Process each colored element
        coloredElements.forEach(element => {
            // Only process elements with content
            if (!element.textContent.trim()) return;
            
            const colorAttr = element.getAttribute('data-color');
            if (!colorAttr) return;
            
            // Get the RGB color
            const [r, g, b] = colorAttr.split(',').map(Number);
            
            // Find the element's position in the overall text
            const range = document.createRange();
            range.selectNodeContents(element);
            
            // Get all text nodes in the editor
            const walker = document.createTreeWalker(
                this.richTextEditor,
                NodeFilter.SHOW_TEXT,
                null,
                false
            );
            
            let totalTextLength = 0;
            let foundStart = false;
            let segmentStart = 0;
            
            // Walk through all text nodes
            while (walker.nextNode()) {
                const textNode = walker.currentNode;
                
                // Check if this text node is inside our element
                if (element.contains(textNode)) {
                    if (!foundStart) {
                        segmentStart = totalTextLength;
                        foundStart = true;
                    }
                }
                
                if (!foundStart) {
                    totalTextLength += textNode.textContent.length;
                }
            }
            
            // If we found the position, add a segment
            if (foundStart) {
                const segment = {
                    start: segmentStart,
                    end: segmentStart + element.textContent.length,
                    color: [r, g, b]
                };
                
                this.textSegments.push(segment);
            }
        });
        
        // Sort segments by start position
        this.textSegments.sort((a, b) => a.start - b.start);
        
        // Fix any overlapping segments
        this.fixOverlappingSegments(this.textSegments, textContent);
    }

    // Helper method to apply colored segments to the editor
    applyColoredSegmentsToEditor(text, segments) {
        this.richTextEditor.innerHTML = ''; // Clear editor
        
        // Sort segments by start position
        const sortedSegments = [...segments].sort((a, b) => a.start - b.start);
        
        // Check for overlapping segments and fix them
        this.fixOverlappingSegments(sortedSegments, text);
        
        // Check if this appears to be rainbow letters (segments of length 1)
        const hasRainbowLetters = this.detectRainbowLetters(sortedSegments, text);
        
        // Check if this appears to be rainbow words
        const hasRainbowWords = this.detectRainbowWords(sortedSegments, text);
        
        let lastEnd = 0;
        
        // Apply each segment
        sortedSegments.forEach(segment => {
            // Text before this segment
            if (segment.start > lastEnd) {
                const beforeText = text.substring(lastEnd, segment.start);
                this.richTextEditor.appendChild(document.createTextNode(beforeText));
            }
            
            // Skip segments that start after the end of the text
            if (segment.start >= text.length) {
                return;
            }
            
            // Adjust end if it goes beyond text length
            const adjustedEnd = Math.min(segment.end, text.length);
            
            // The colored segment
            const segmentText = text.substring(segment.start, adjustedEnd);
            const span = document.createElement('span');
            
            // Determine the appropriate classes
            let className = 'colored-text';
            
            // Add special classes for rainbow letters and words
            if (hasRainbowLetters && segmentText.length === 1 && !/\s/.test(segmentText)) {
                className += ' word-letter rainbow-letter';
                span.setAttribute('data-letter-style', 'rainbow');
            } else if (hasRainbowWords && /^\S+$/.test(segmentText)) {
                className += ' rainbow-word';
                span.setAttribute('data-word-style', 'rainbow');
            }
            
            span.className = className;
            span.style.color = `rgb(${segment.color[0]},${segment.color[1]},${segment.color[2]})`;
            span.setAttribute('data-color', segment.color.join(','));
            span.textContent = segmentText;
            this.richTextEditor.appendChild(span);
            
            lastEnd = adjustedEnd;
        });
        
        // Text after the last segment
        if (lastEnd < text.length) {
            const afterText = text.substring(lastEnd);
            this.richTextEditor.appendChild(document.createTextNode(afterText));
        }
        
        // If we have rainbow formatting, ensure the CSS is in place
        if (hasRainbowLetters || hasRainbowWords) {
            this.ensureRainbowStyles();
        }
    }

    // Helper to fix overlapping segments
    fixOverlappingSegments(segments, text) {
        // Find and fix any overlapping segments
        for (let i = 0; i < segments.length - 1; i++) {
            const current = segments[i];
            const next = segments[i + 1];
            
            // Check if segments overlap
            if (current.end > next.start) {
                // Fix by truncating the first segment
                current.end = next.start;
                
                // If this makes the segment invalid, remove it
                if (current.end <= current.start) {
                    segments.splice(i, 1);
                    i--; // Adjust index since we removed an element
                }
            }
        }
        
        // Remove any segments with invalid boundaries
        for (let i = segments.length - 1; i >= 0; i--) {
            const segment = segments[i];
            
            // Check for out-of-bounds segments
            if (segment.start < 0 || segment.end > text.length || segment.start >= segment.end) {
                segments.splice(i, 1);
            }
        }
    }

    // New helper method to detect rainbow letter formatting
    detectRainbowLetters(segments, text) {
        // Count how many segments are single letters
        let singleLetterCount = 0;
        let nonLetterCount = 0;
        
        segments.forEach(segment => {
            const segmentText = text.substring(segment.start, segment.end);
            if (segmentText.length === 1 && !/\s/.test(segmentText)) {
                singleLetterCount++;
            } else {
                nonLetterCount++;
            }
        });
        
        // If more than 70% of segments are single letters, it's likely rainbow letters
        return singleLetterCount > 0 && (singleLetterCount / (singleLetterCount + nonLetterCount)) > 0.7;
    }

    // New helper method to detect rainbow word formatting
    detectRainbowWords(segments, text) {
        // If there are no segments or only a few, it's probably not rainbow words
        if (segments.length < 3) return false;
        
        // Check if segments appear to be words
        let wordCount = 0;
        let nonWordCount = 0;
        
        segments.forEach(segment => {
            const segmentText = text.substring(segment.start, segment.end);
            // A segment is a "word" if it contains no whitespace and is longer than 1 char
            if (/^\S+$/.test(segmentText) && segmentText.length > 1) {
                wordCount++;
            } else if (segmentText.length > 1) { // Ignore single chars for this count
                nonWordCount++;
            }
        });
        
        // If more than 70% of multi-char segments are words, it's likely rainbow words
        return wordCount > 0 && (wordCount / (wordCount + nonWordCount)) > 0.7;
    }

    // Add helper to ensure rainbow styles are in the document
    ensureRainbowStyles() {
        // Check if styles are already present
        const existingStyle = document.getElementById('rainbow-text-styles');
        if (existingStyle) return;
        
        // Create styles if not present
        const styleElement = document.createElement('style');
        styleElement.id = 'rainbow-text-styles';
        
        styleElement.textContent = `
            .rainbow-word, .word-letter, .rainbow-letter {
                display: inline;
                white-space: pre;
                margin: 0;
                padding: 0;
                border-spacing: 0;
                font-size: inherit;
                line-height: inherit;
            }
            
            #richTextEditor {
                white-space: pre-wrap;
                word-break: break-word;
            }
        `;
        document.head.appendChild(styleElement);
    }

    // Helper to normalize selection by removing rainbow spans
    normalizeSelectionRemovingRainbow(range) {
        if (!range) return;
        
        // Get elements fully or partially within the selection
        const coloredElements = this.richTextEditor.querySelectorAll('.colored-text, .rainbow-word, .rainbow-letter');
        
        // Find all elements that intersect with our selection
        const elementsToProcess = [];
        for (const element of coloredElements) {
            if (range.intersectsNode(element)) {
                elementsToProcess.push(element);
            }
        }
        
        // Process each element that intersects with our selection
        for (const element of elementsToProcess) {
            // Skip non-span elements or elements outside our editor
            if (!this.richTextEditor.contains(element)) continue;
            
            // If the element is fully contained in the selection, replace it with its text
            if (range.comparePoint(element, 0) <= 0 && 
                range.comparePoint(element, element.childNodes.length) >= 0) {
                if (element.parentNode) {
                    const textNode = document.createTextNode(element.textContent);
                    element.parentNode.replaceChild(textNode, element);
                }
            }
        }
        
        // Restore selection
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
    }

    // Helper to save selection state
    saveSelection() {
        if (window.getSelection) {
            const sel = window.getSelection();
            if (sel.getRangeAt && sel.rangeCount) {
                return sel.getRangeAt(0).cloneRange();
            }
        }
        return null;
    }

    // Helper to restore selection
    restoreSelection(range) {
        if (range) {
            if (window.getSelection) {
                const sel = window.getSelection();
                sel.removeAllRanges();
                sel.addRange(range);
            }
        }
    }

    // Helper to place cursor after a node
    placeCursorAfterNode(node) {
        const selection = window.getSelection();
        const range = document.createRange();
        range.setStartAfter(node);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
    }

    // Helper to update color picker based on cursor position
    updateColorPickerFromCursor() {
        const selection = window.getSelection();
        if (!selection.rangeCount) return;
        
        const range = selection.getRangeAt(0);
        
        // Only process if cursor is within our editor
        if (!this.richTextEditor.contains(range.commonAncestorContainer)) return;
        
        // Get the color at cursor position
        const cursorColor = this.getColorAtCursor(range);
        if (cursorColor) {
            // Update both model and UI
            this.forceColorPickerUpdate(cursorColor);
        }
    }

    // Helper to get color at cursor position
    getColorAtCursor(range) {
        // Get the element at cursor position
        let currentNode = range.startContainer;
        let parentElement = null;
        
        // If it's a text node, get its parent
        if (currentNode.nodeType === Node.TEXT_NODE) {
            parentElement = currentNode.parentNode;
        } else {
            parentElement = currentNode;
        }
        
        // Look for the closest colored element
        let coloredElement = null;
        let current = parentElement;
        
        while (current && current.id !== this.options.editorId) {
            if (current.hasAttribute('data-color') || 
                current.style && current.style.color ||
                current.getAttribute('color')) {
                coloredElement = current;
                break;
            }
            current = current.parentNode;
        }
        
        if (coloredElement) {
            // Try to get color from our data attribute first
            const dataColor = coloredElement.getAttribute('data-color');
            if (dataColor) {
                const [r, g, b] = dataColor.split(',').map(Number);
                return this.rgbToHex(r, g, b);
            }
            
            // Try to get from style
            if (coloredElement.style && coloredElement.style.color) {
                const colorStr = coloredElement.style.color;
                // Parse rgb(...) format
                const rgbMatch = colorStr.match(/rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/);
                if (rgbMatch) {
                    return this.rgbToHex(parseInt(rgbMatch[1]), parseInt(rgbMatch[2]), parseInt(rgbMatch[3]));
                }
                
                // If it's already a hex value
                if (colorStr.startsWith('#')) {
                    return colorStr;
                }
            }
            
            // Try font color attribute
            const fontColor = coloredElement.getAttribute('color');
            if (fontColor) {
                return fontColor;
            }
        }
        
        // If no color found, return the default text color
        return this.textColorHex;
    }

    // Helper to force color picker update
    forceColorPickerUpdate(hexColor) {
        // Update our model
        this.textColorHex = hexColor;
        
        // Update the DOM
        const colorInput = document.querySelector('input[type="color"]');
        if (colorInput) {
            colorInput.value = hexColor;
        }
    }

    // Helper to find the current word/span at cursor position
    getCurrentWordOrSpan(range) {
        // First, check if cursor is inside an existing span
        let currentNode = range.startContainer;
        
        // If we're directly in a colored span element
        if (currentNode.nodeType === Node.ELEMENT_NODE && 
            (currentNode.classList.contains('colored-text') || 
             currentNode.classList.contains('rainbow-word') || 
             currentNode.classList.contains('word-letter') ||
             currentNode.classList.contains('rainbow-letter'))) {
            return currentNode;
        }
        
        // Check if we're in a text node inside a colored element
        if (currentNode.nodeType === Node.TEXT_NODE) {
            let parent = currentNode.parentNode;
            if (parent && parent !== this.richTextEditor && 
                (parent.classList.contains('colored-text') || 
                 parent.classList.contains('rainbow-word') || 
                 parent.classList.contains('word-letter') ||
                 parent.classList.contains('rainbow-letter'))) {
                return parent;
            }
            
            // If not in a colored element, try to find a word
            const text = currentNode.nodeValue;
            const offset = range.startOffset;
            
            // Find word boundaries
            let start = offset;
            let end = offset;
            
            // Search backward for start of word
            while (start > 0 && !/\s/.test(text[start - 1])) {
                start--;
            }
            
            // Search forward for end of word
            while (end < text.length && !/\s/.test(text[end])) {
                end++;
            }
            
            // If we found a valid word
            if (start < end && end - start > 0) {
                // Create a range for the word
                const wordRange = document.createRange();
                wordRange.setStart(currentNode, start);
                wordRange.setEnd(currentNode, end);
                
                // Select the word - this helps with the delete operation
                const selection = window.getSelection();
                selection.removeAllRanges();
                selection.addRange(wordRange);
                
                return null; // Return null to indicate we've set up a selection instead
            }
        }
        
        // Check if any parent is a colored element
        let element = currentNode;
        while (element && element.id !== this.options.editorId) {
            if (element.classList && 
                (element.classList.contains('colored-text') || 
                 element.classList.contains('rainbow-word') || 
                 element.classList.contains('word-letter') ||
                 element.classList.contains('rainbow-letter'))) {
                return element; // Found a span containing the cursor
            }
            element = element.parentNode;
        }
        
        return null; // No suitable span or word found
    }

    // RGB to Hex conversion
    rgbToHex(r, g, b) {
        return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1);
    }

    // Hex to RGB conversion
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
    }
}

// Export the class for use in other files
window.TextEditor = TextEditor; 