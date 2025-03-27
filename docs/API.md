# LED Sign Controller API Documentation

This document describes the API endpoints available for the LED sign controller application.

## Table of Contents
- [Playlist Management](#playlist-management)
  - [Get All Playlist Items](#get-all-playlist-items)
  - [Create Playlist Item](#create-playlist-item)
  - [Get Specific Playlist Item](#get-specific-playlist-item)
  - [Update Playlist Item](#update-playlist-item)
  - [Delete Playlist Item](#delete-playlist-item)
  - [Reorder Playlist Items](#reorder-playlist-items)
- [Settings](#settings)
  - [Get Brightness](#get-brightness)
  - [Update Brightness](#update-brightness)
- [Preview Mode](#preview-mode)
  - [Start Preview Mode](#start-preview-mode)
  - [Update Preview Content](#update-preview-content)
  - [Exit Preview Mode](#exit-preview-mode)
  - [Check Preview Status](#check-preview-status)
  - [Ping Preview Session](#ping-preview-session)
  - [Check Session Ownership](#check-session-ownership)
- [Real-time Events](#real-time-events)
  - [Brightness Events](#brightness-events)
  - [Editor Lock Events](#editor-lock-events)
  - [Playlist Events](#playlist-events)
- [JavaScript SDK Examples](#javascript-sdk-examples)
  - [Preview Mode Flow](#preview-mode-flow)
  - [Editor Lock Management](#editor-lock-management)
  - [Real-time Event Handling](#real-time-event-handling)

## Playlist Management

### Get All Playlist Items

Retrieves all items in the playlist.

- **URL**: `/api/playlist/items`
- **Method**: `GET`
- **Response**: Array of playlist items
  
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "duration": 10,
    "repeat_count": null,
    "border_effect": { "Rainbow": null },
    "content": {
      "content_type": "Text",
      "data": {
        "text": "Hello World",
        "scroll": false,
        "color": [255, 255, 255],
        "speed": 50.0,
        "text_segments": null
      }
    }
  }
]
```

### Create Playlist Item

Creates a new playlist item.

- **URL**: `/api/playlist/items`
- **Method**: `POST`
- **Body**: Playlist item (ID will be generated if not provided)
- **Response**: Created playlist item with ID

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "duration": 10,
  "repeat_count": null,
  "border_effect": { "Rainbow": null },
  "content": {
    "content_type": "Text",
    "data": {
      "text": "Hello World",
      "scroll": false,
      "color": [255, 255, 255],
      "speed": 50.0,
      "text_segments": null
    }
  }
}
```

### Get Specific Playlist Item

Retrieves a specific playlist item by ID.

- **URL**: `/api/playlist/items/:id`
- **Method**: `GET`
- **Response**: Playlist item
- **Error Codes**: 
  - `404` - Item not found

### Update Playlist Item

Updates a specific playlist item.

- **URL**: `/api/playlist/items/:id`
- **Method**: `PUT`
- **Body**: Updated playlist item
- **Response**: Updated playlist item
- **Error Codes**:
  - `404` - Item not found

### Delete Playlist Item

Deletes a specific playlist item.

- **URL**: `/api/playlist/items/:id`
- **Method**: `DELETE`
- **Response**: Status code only
- **Error Codes**:
  - `404` - Item not found

### Reorder Playlist Items

Reorders all playlist items.

- **URL**: `/api/playlist/reorder`
- **Method**: `PUT`
- **Body**: Ordered array of item IDs
```json
{
  "item_ids": ["id1", "id2", "id3"]
}
```
- **Response**: Reordered list of playlist items
- **Error Codes**:
  - `400` - Invalid reorder request (missing items or incorrect count)

## Settings

### Get Brightness

Retrieves the current brightness setting.

- **URL**: `/api/settings/brightness`
- **Method**: `GET`
- **Response**: Current brightness (0-100)
```json
{
  "brightness": 75
}
```

### Update Brightness

Updates the display brightness.

- **URL**: `/api/settings/brightness`
- **Method**: `PUT`
- **Body**: New brightness setting
```json
{
  "brightness": 75
}
```
- **Response**: Updated brightness setting
```json
{
  "brightness": 75
}
```

## Preview Mode

### Start Preview Mode

Starts preview mode with the specified content.

- **URL**: `/api/preview`
- **Method**: `POST`
- **Body**: Playlist item to preview
- **Response**: Preview mode response with session ID
```json
{
  "item": {
    "id": "preview-item",
    "duration": 10,
    "border_effect": null,
    "content": {
      "content_type": "Text",
      "data": {
        "text": "Preview Text",
        "scroll": false,
        "color": [255, 255, 255],
        "speed": 50.0,
        "text_segments": null
      }
    }
  },
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Note**: The session ID returned must be saved and used for all subsequent preview operations.

### Update Preview Content

Updates the content being previewed.

- **URL**: `/api/preview`
- **Method**: `PUT`
- **Body**: Updated item and session ID
```json
{
  "item": {
    "id": "preview-item",
    "duration": 10,
    "border_effect": null,
    "content": {
      "content_type": "Text",
      "data": {
        "text": "Updated Preview Text",
        "scroll": false,
        "color": [255, 0, 0],
        "speed": 50.0,
        "text_segments": null
      }
    }
  },
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
- **Response**: Updated preview response
- **Error Codes**:
  - `403` - Session does not own the preview lock
  - `404` - Not in preview mode

### Exit Preview Mode

Exits preview mode.

- **URL**: `/api/preview`
- **Method**: `DELETE`
- **Body**: Session ID for authorization
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
- **Response**: Status code only
- **Error Codes**:
  - `403` - Session does not own the preview lock
  - `404` - Not in preview mode

**Note**: Only the session that started preview mode can exit it.

### Check Preview Status

Checks if the display is currently in preview mode.

- **URL**: `/api/preview/status`
- **Method**: `GET`
- **Response**: Preview mode state
```json
{
  "active": true
}
```

### Ping Preview Session

Prevents the preview mode from timing out. Only the session that started the preview can ping it.

- **URL**: `/api/preview/ping`
- **Method**: `POST`
- **Body**: Session ID for authorization
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
- **Response**: Status code only
- **Error Codes**:
  - `403` - Session does not own the preview lock
  - `404` - Not in preview mode 

### Check Session Ownership

Checks if a session owns the current preview lock.

- **URL**: `/api/preview/session`
- **Method**: `POST`
- **Body**: Session ID to check
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```
- **Response**: Ownership status
```json
{
  "is_owner": true
}
```

## Real-time Events

The application provides Server-Sent Events (SSE) for real-time updates.

### Brightness Events

Subscribe to brightness change events.

- **URL**: `/api/events/brightness`
- **Method**: `GET`
- **Content Type**: `text/event-stream`
- **Event Format**:
```json
{
  "brightness": 75
}
```

### Editor Lock Events

Subscribe to editor lock status changes.

- **URL**: `/api/events/editor`
- **Method**: `GET`
- **Content Type**: `text/event-stream`
- **Event Format**:
```json
{
  "locked": true,
  "locked_by": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Playlist Events

Subscribe to playlist update events.

- **URL**: `/api/events/playlist`
- **Method**: `GET`
- **Content Type**: `text/event-stream`
- **Event Format**:
```json
{
  "items": [/* array of playlist items */],
  "action": "Add" // One of: "Add", "Update", "Delete", "Reorder"
}
```

## JavaScript SDK Examples

### Preview Mode Flow

Complete example of starting, updating, and exiting preview mode with proper session management:

```javascript
// Start a preview session
async function startPreview(content) {
  const response = await fetch('/api/preview', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(content)
  });
  
  if (!response.ok) throw new Error('Failed to start preview');
  
  const data = await response.json();
  return data.session_id; // Save this for future operations
}

// Update the preview content
async function updatePreview(content, sessionId) {
  const response = await fetch('/api/preview', {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      item: content,
      session_id: sessionId
    })
  });
  
  if (!response.ok) {
    if (response.status === 403) {
      throw new Error('Session does not own the preview lock');
    }
    throw new Error('Failed to update preview');
  }
  
  return await response.json();
}

// Exit preview mode
async function exitPreview(sessionId) {
  const response = await fetch('/api/preview', {
    method: 'DELETE',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      session_id: sessionId
    })
  });
  
  if (!response.ok) {
    if (response.status === 403) {
      throw new Error('Session does not own the preview lock');
    }
    throw new Error('Failed to exit preview');
  }
}

// Keep preview active with pings
function setupPreviewPings(intervalMs = 3000) {
  const pingInterval = setInterval(async () => {
    try {
      const response = await fetch('/api/preview/ping', {
        method: 'POST'
      });
      
      if (!response.ok) {
        console.warn('Preview session may have expired');
        clearInterval(pingInterval);
      }
    } catch (err) {
      console.error('Error pinging preview:', err);
      clearInterval(pingInterval);
    }
  }, intervalMs);
  
  return pingInterval; // Store this to clear it when done
}

// Usage example
async function previewWorkflow() {
  // Example content to preview
  const content = {
    id: "temp-preview",
    duration: 10,
    content: {
      content_type: "Text",
      data: {
        text: "Preview Text",
        scroll: false,
        color: [255, 255, 255],
        speed: 50.0
      }
    }
  };
  
  try {
    // Start preview and get session ID
    const sessionId = await startPreview(content);
    console.log('Preview started with session:', sessionId);
    
    // Start pinging to keep preview active
    const pingInterval = setupPreviewPings();
    
    // Update preview after 5 seconds
    setTimeout(async () => {
      try {
        content.content.data.text = "Updated Text";
        content.content.data.color = [255, 0, 0];
        await updatePreview(content, sessionId);
        console.log('Preview updated');
      } catch (err) {
        console.error('Error updating preview:', err);
      }
    }, 5000);
    
    // Exit preview after 10 seconds
    setTimeout(async () => {
      try {
        clearInterval(pingInterval); // Stop pings
        await exitPreview(sessionId);
        console.log('Preview exited');
      } catch (err) {
        console.error('Error exiting preview:', err);
      }
    }, 10000);
    
  } catch (err) {
    console.error('Preview workflow error:', err);
  }
}
```

### Editor Lock Management

Example for handling editor locks with SSE:

```javascript
function setupEditorLockMonitoring(onLockChanged) {
  const eventSource = new EventSource('/api/events/editor');
  
  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    onLockChanged(data.locked, data.locked_by);
  };
  
  // Handle connection errors
  eventSource.onerror = (err) => {
    console.error('Editor lock SSE error:', err);
    eventSource.close();
    
    // Attempt to reconnect after a delay
    setTimeout(() => setupEditorLockMonitoring(onLockChanged), 5000);
  };
  
  return {
    close: () => eventSource.close()
  };
}

// Check if the current session owns the lock
async function checkSessionLockOwnership(sessionId) {
  const response = await fetch('/api/preview/session', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      session_id: sessionId
    })
  });
  
  if (!response.ok) throw new Error('Failed to check session ownership');
  
  const data = await response.json();
  return data.is_owner;
}

// Usage example
let currentSessionId = null;

// Setup UI state based on lock
function updateUIForLockState(isLocked, lockOwner) {
  const editButton = document.getElementById('edit-button');
  
  if (isLocked) {
    editButton.disabled = true;
    editButton.textContent = 'Editor Locked';
    
    // Check if we own the lock
    if (currentSessionId && lockOwner === currentSessionId) {
      editButton.disabled = false;
      editButton.textContent = 'Continue Editing';
    }
  } else {
    editButton.disabled = false;
    editButton.textContent = 'Edit Content';
    currentSessionId = null;
  }
}

// Start monitoring
const lockMonitor = setupEditorLockMonitoring(updateUIForLockState);

// When starting edit session, save the session ID
async function onEditButtonClick() {
  if (currentSessionId) {
    // We're resuming our own edit session
    const isOwner = await checkSessionLockOwnership(currentSessionId);
    if (!isOwner) {
      alert('Your edit session has expired');
      currentSessionId = null;
      return;
    }
  } else {
    // Start a new edit session
    try {
      const content = {}; // Get current content
      const response = await startPreview(content);
      currentSessionId = response.session_id;
      setupPreviewPings();
    } catch (err) {
      alert('Cannot start editing: ' + err.message);
    }
  }
  
  // Open editor UI...
}
```

### Real-time Event Handling

Example for handling brightness and playlist updates:

```javascript
// Monitor brightness changes
function setupBrightnessMonitoring(onBrightnessChanged) {
  const eventSource = new EventSource('/api/events/brightness');
  
  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    onBrightnessChanged(data.brightness);
  };
  
  eventSource.onerror = (err) => {
    console.error('Brightness SSE error:', err);
    eventSource.close();
    
    // Attempt to reconnect after a delay
    setTimeout(() => setupBrightnessMonitoring(onBrightnessChanged), 5000);
  };
  
  return {
    close: () => eventSource.close()
  };
}

// Monitor playlist changes
function setupPlaylistMonitoring(onPlaylistChanged) {
  const eventSource = new EventSource('/api/events/playlist');
  
  eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    onPlaylistChanged(data.items, data.action);
  };
  
  eventSource.onerror = (err) => {
    console.error('Playlist SSE error:', err);
    eventSource.close();
    
    // Attempt to reconnect after a delay
    setTimeout(() => setupPlaylistMonitoring(onPlaylistChanged), 5000);
  };
  
  return {
    close: () => eventSource.close()
  };
}

// Usage example
let brightnessSlider = document.getElementById('brightness-slider');
let playlistContainer = document.getElementById('playlist-items');

// Update UI when brightness changes remotely
function onBrightnessChanged(brightness) {
  brightnessSlider.value = brightness;
  console.log(`Brightness updated to ${brightness}%`);
}

// Update UI when playlist changes remotely
function onPlaylistChanged(items, action) {
  console.log(`Playlist ${action} detected with ${items.length} items`);
  
  // Refresh playlist display
  playlistContainer.innerHTML = '';
  items.forEach(item => {
    const element = document.createElement('div');
    element.textContent = getItemDisplayName(item);
    playlistContainer.appendChild(element);
  });
}

// When user changes brightness
brightnessSlider.addEventListener('change', async () => {
  const brightness = parseInt(brightnessSlider.value);
  
  try {
    await fetch('/api/settings/brightness', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ brightness })
    });
  } catch (err) {
    console.error('Failed to update brightness:', err);
  }
});

// Start monitoring
const brightnessMonitor = setupBrightnessMonitoring(onBrightnessChanged);
const playlistMonitor = setupPlaylistMonitoring(onPlaylistChanged);
```