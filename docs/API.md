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

Starts preview mode with the specified content. Will fail if another preview session is already active.

- **URL**: `/api/preview`
- **Method**: `POST`
- **Body**: Playlist item to preview (no session ID needed)
```json
{
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
}
```
- **Response**: Preview mode response with server-generated session ID
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
- **Error Codes**:
  - `403` - Another preview session is already active

**Note**: The session ID returned must be saved and used for all subsequent preview operations (update, ping, exit). 

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
