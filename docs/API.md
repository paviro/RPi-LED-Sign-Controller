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
- [Content Payloads](#content-payloads)
  - [Text Content](#text-content)
  - [Image Content](#image-content)
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
- [Image Library](#image-library)
  - [Upload Image](#upload-image)
  - [Fetch Image](#fetch-image)
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
      "type": "Text",
      "data": {
        "type": "Text",
        "text": "Hello World",
        "scroll": false,
        "color": [255, 255, 255],
        "speed": 50.0,
        "text_segments": null
      }
    }
  },
  {
    "id": "44dc1488-be53-4d2d-b6b8-30c4fee522e8",
    "duration": null,
    "repeat_count": 3,
    "border_effect": null,
    "content": {
      "type": "Image",
      "data": {
        "type": "Image",
        "image_id": "c3c8d980-27a7-4a7a-9f56-1f4b1f8bb0fc",
        "natural_width": 128,
        "natural_height": 64,
        "transform": { "x": 0, "y": 0, "scale": 1 },
        "animation": {
          "keyframes": [
            { "timestamp_ms": 0, "x": 0, "y": 0, "scale": 1 },
            { "timestamp_ms": 2000, "x": -16, "y": 0, "scale": 1.5 }
          ],
          "iterations": null
        }
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
    "type": "Text",
    "data": {
      "type": "Text",
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

## Content Payloads

Every playlist or preview item contains a `content` object. The outer `content.type` helps the UI/editor know which tool to render, while the nested `content.data` is a tagged union that repeats the `type` field and carries the actual properties for that content kind.

### Text Content

Text payloads are identical to the original implementation but now live inside `content.data`.

- `text` - Raw UTF-8 text
- `scroll` - When `true`, the message scrolls and you must provide `repeat_count` instead of `duration`
- `color` - Base RGB color triplet
- `speed` - Scroll speed (0-100)
- `text_segments` - Optional overrides for colors/formatting (see frontend docs)

Static text (`scroll: false`) requires `duration` and must omit `repeat_count`. Scrolling text requires `repeat_count` and must omit `duration`.

```json
"content": {
  "type": "Text",
  "data": {
    "type": "Text",
    "text": "Welcome!",
    "scroll": true,
    "color": [255, 255, 255],
    "speed": 50,
    "text_segments": [
      { "start": 0, "end": 7, "color": [255, 0, 0] }
    ]
  }
}
```

### Image Content

Images can be static or animated. Upload images via `POST /api/images` to obtain an `image_id`. The backend stores the binary PNG under `/var/lib/led-matrix-controller/images`, and playlist items simply reference that ID.

- `image_id` - UUID returned by the upload endpoint
- `natural_width` / `natural_height` - Source dimensions so the editor can scale accurately
- `transform` - `{ "x": number, "y": number, "scale": number }` describing how the bitmap is positioned relative to the panel's top-left corner
- `animation` *(optional)* - Keyframe animation with at least two entries when present
  - `keyframes` - Each entry has `timestamp_ms`, `x`, `y`, and `scale`
  - `iterations` - Number of loops (`null` = infinite)

Static images require `duration` and must omit `repeat_count`. Animated images (two or more keyframes) require `repeat_count`, must omit `duration`, and the frontend enforces the minimum keyframe count.

```json
"content": {
  "type": "Image",
  "data": {
    "type": "Image",
    "image_id": "c3c8d980-27a7-4a7a-9f56-1f4b1f8bb0fc",
    "natural_width": 128,
    "natural_height": 64,
    "transform": { "x": -8, "y": 0, "scale": 1.25 },
    "animation": {
      "keyframes": [
        { "timestamp_ms": 0, "x": 0, "y": 0, "scale": 1 },
        { "timestamp_ms": 2500, "x": -16, "y": 0, "scale": 1.5 }
      ],
      "iterations": null
    }
  }
}
```

Set `"animation": null` (or omit it) to display a static image with a fixed transform.

### Clock Content

Clock entries render the Raspberry Pi's local time centered on the display. They always use `duration` for timing and must omit `repeat_count`.

- `format` - `"24h"` or `"12h"`
- `show_seconds` - `true` to update every second, `false` for minutes only
- `color` - RGB tuple for the digits

```json
"content": {
  "type": "Clock",
  "data": {
    "type": "Clock",
    "format": "24h",
    "show_seconds": false,
    "color": [255, 255, 255]
  }
}
```

Clock items support the same border effects as other playlist entries.

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
    "type": "Text",
    "data": {
      "type": "Text",
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
      "type": "Text",
      "data": {
        "type": "Text",
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
      "type": "Text",
      "data": {
        "type": "Text",
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

## Image Library

Upload an image once and reference it across multiple playlist items via the returned `image_id`.

### Upload Image

Accepts multipart uploads, validates the payload, converts everything to PNG, and stores the bytes under `/var/lib/led-matrix-controller/images`.

- **URL**: `/api/images`
- **Method**: `POST`
- **Body**: `multipart/form-data` with a single `file` field (PNG/JPEG/GIF, max 30 MB)
- **Response**:
```json
{
  "image_id": "c3c8d980-27a7-4a7a-9f56-1f4b1f8bb0fc",
  "width": 128,
  "height": 64,
  "thumbnail_width": 64,
  "thumbnail_height": 48
}
```
- **Error Codes**:
  - `400` - Invalid multipart payload or empty file
  - `413` - File exceeds 30 MB
  - `415` - Unsupported image format/decoder failure
  - `500` - Failed to persist the PNG

### Fetch Image

Returns the stored PNG bytes for previews or diagnostics.

- **URL**: `/api/images/:id`
- **Method**: `GET`
- **Response**: Raw `image/png` body (use as-is in `<img>` tags or `<canvas>`)
- **Error Codes**:
  - `404` - No image exists for that `image_id`

### Fetch Image Thumbnail

Returns a pre-generated thumbnail (PNG) for lightweight previews such as playlist cards. Thumbnails are generated automatically during upload and lazily regenerated on demand if missing.

- **URL**: `/api/images/:id/thumbnail`
- **Method**: `GET`
- **Response**: Raw `image/png` thumbnail (fits within 128×96 while preserving aspect ratio)
- **Error Codes**:
  - `404` - No image exists for that `image_id`

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
