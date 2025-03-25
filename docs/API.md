# LED Matrix Controller API Documentation

This document outlines all available API endpoints for the LED Matrix Controller.

## Table of Contents

- [Playlist Management](#playlist-management)
  - [Get All Playlist Items](#get-all-playlist-items)
  - [Create Playlist Item](#create-playlist-item)
  - [Get Playlist Item](#get-playlist-item)
  - [Update Playlist Item](#update-playlist-item)
  - [Delete Playlist Item](#delete-playlist-item)
  - [Reorder Playlist Items](#reorder-playlist-items)
- [Settings](#settings)
  - [Get Brightness](#get-brightness)
  - [Update Brightness](#update-brightness)
- [Preview Mode](#preview-mode)
  - [Start Preview Mode](#start-preview-mode)
  - [Exit Preview Mode](#exit-preview-mode)
  - [Get Preview Mode Status](#get-preview-mode-status)
  - [Ping Preview Mode](#ping-preview-mode)
- [Data Structures](#data-structures)
  - [DisplayContent](#displaycontent)
  - [ContentType](#contenttype)
  - [ContentData](#contentdata)
  - [TextContent](#textcontent)
  - [BorderEffect](#bordereffect)
  - [TextSegment](#textsegment)
  - [TextFormatting](#textformatting)

## Playlist Management

### Get All Playlist Items

Retrieves all items in the playlist.

- **URL**: `/api/playlist/items`
- **Method**: `GET`
- **Authentication**: None

#### Success Response

- **Code**: 200 OK
- **Content**: Array of [DisplayContent](#displaycontent) objects

```json
[
  {
    "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "repeat_count": 1,
    "border_effect": {"Rainbow": null},
    "content": {
      "type": "Text",
      "data": {
        "text": "Welcome to LED Matrix Controller",
        "scroll": true,
        "color": [255, 255, 255],
        "speed": 50.0,
        "text_segments": null
      }
    }
  },
  ...
]
```

### Create Playlist Item

Creates a new playlist item.

- **URL**: `/api/playlist/items`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Authentication**: None
- **Request Body**: [DisplayContent](#displaycontent) object

#### Notes

- The `id` field is optional and will be automatically generated if omitted.

#### Success Response

- **Code**: 201 Created
- **Content**: The created [DisplayContent](#displaycontent) object with assigned ID

```json
{
  "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "repeat_count": 1,
  "border_effect": {"Rainbow": null},
  "content": {
    "type": "Text",
    "data": {
      "text": "Welcome to LED Matrix Controller",
      "scroll": true,
      "color": [255, 255, 255],
      "speed": 50.0,
      "text_segments": null
    }
  }
}
```

### Get Playlist Item

Retrieves a specific playlist item by ID.

- **URL**: `/api/playlist/items/:id`
- **Method**: `GET`
- **Authentication**: None
- **URL Parameters**: 
  - `id`: UUID of the playlist item

#### Success Response

- **Code**: 200 OK
- **Content**: [DisplayContent](#displaycontent) object

```json
{
  "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "repeat_count": 1,
  "border_effect": {"Rainbow": null},
  "content": {
    "type": "Text",
    "data": {
      "text": "Welcome to LED Matrix Controller",
      "scroll": true,
      "color": [255, 255, 255],
      "speed": 50.0,
      "text_segments": null
    }
  }
}
```

#### Error Response

- **Code**: 404 Not Found
- **Content**: None

### Update Playlist Item

Updates a specific playlist item by ID.

- **URL**: `/api/playlist/items/:id`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **Authentication**: None
- **URL Parameters**: 
  - `id`: UUID of the playlist item
- **Request Body**: [DisplayContent](#displaycontent) object

#### Notes

- The `id` in the URL must match an existing playlist item
- The `id` in the request body is ignored; the URL parameter takes precedence

#### Success Response

- **Code**: 200 OK
- **Content**: The updated [DisplayContent](#displaycontent) object

```json
{
  "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
  "repeat_count": 1,
  "border_effect": {"Rainbow": null},
  "content": {
    "type": "Text",
    "data": {
      "text": "Updated Text Content",
      "scroll": true,
      "color": [255, 255, 255],
      "speed": 50.0,
      "text_segments": null
    }
  }
}
```

#### Error Response

- **Code**: 404 Not Found
- **Content**: None

### Delete Playlist Item

Deletes a specific playlist item by ID.

- **URL**: `/api/playlist/items/:id`
- **Method**: `DELETE`
- **Authentication**: None
- **URL Parameters**: 
  - `id`: UUID of the playlist item

#### Success Response

- **Code**: 200 OK
- **Content**: None

#### Error Response

- **Code**: 404 Not Found
- **Content**: None

### Reorder Playlist Items

Changes the order of playlist items.

- **URL**: `/api/playlist/reorder`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **Authentication**: None
- **Request Body**:

```json
{
  "item_ids": ["id1", "id2", "id3", ...]
}
```

#### Notes

- The `item_ids` array must contain all existing playlist item IDs
- The order of IDs in the array determines the new order of playlist items

#### Success Response

- **Code**: 200 OK
- **Content**: Array of reordered [DisplayContent](#displaycontent) objects

```json
[
  {
    "id": "id1",
    "repeat_count": 1,
    "border_effect": null,
    "content": {
      "type": "Text",
      "data": {
        "text": "First item",
        "scroll": true,
        "color": [255, 255, 255],
        "speed": 50.0,
        "text_segments": null
      }
    }
  },
  // Additional items...
]
```

#### Error Response

- **Code**: 400 Bad Request
- **Content**: None

## Settings

### Get Brightness

Retrieves the current brightness setting.

- **URL**: `/api/settings/brightness`
- **Method**: `GET`
- **Authentication**: None

#### Success Response

- **Code**: 200 OK
- **Content**:

```json
{
  "brightness": 100
}
```

### Update Brightness

Updates the brightness setting.

- **URL**: `/api/settings/brightness`
- **Method**: `PUT`
- **Content-Type**: `application/json`
- **Authentication**: None
- **Request Body**:

```json
{
  "brightness": 75
}
```

#### Notes

- Brightness value must be between 0 and 100

#### Success Response

- **Code**: 200 OK
- **Content**: Updated brightness settings

```json
{
  "brightness": 75
}
```

## Preview Mode

Preview mode allows temporarily displaying content without adding it to the playlist.

### Start Preview Mode

Starts preview mode with the provided content.

- **URL**: `/api/preview`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **Authentication**: None
- **Request Body**: [DisplayContent](#displaycontent) object

#### Notes

- The `id` field is optional and will be automatically generated if omitted
- Entering preview mode pauses the regular playlist playback

#### Success Response

- **Code**: 200 OK
- **Content**: The [DisplayContent](#displaycontent) object being previewed

```json
{
  "id": "preview-d290f1ee-6c54-4b01-90e6-d701748f0851",
  "repeat_count": 1,
  "border_effect": {"Rainbow": null},
  "content": {
    "type": "Text",
    "data": {
      "text": "Preview Text",
      "scroll": true,
      "color": [255, 255, 255],
      "speed": 50.0,
      "text_segments": null
    }
  }
}
```

### Exit Preview Mode

Exits preview mode and returns to normal playlist playback.

- **URL**: `/api/preview`
- **Method**: `DELETE`
- **Authentication**: None

#### Success Response

- **Code**: 200 OK
- **Content**: None

### Get Preview Mode Status

Checks if the display is currently in preview mode.

- **URL**: `/api/preview/status`
- **Method**: `GET`
- **Authentication**: None

#### Success Response

- **Code**: 200 OK
- **Content**:

```json
{
  "active": true
}
```

### Ping Preview Mode

Keeps preview mode active (prevents timeout).

- **URL**: `/api/preview/ping`
- **Method**: `POST`
- **Authentication**: None

#### Notes

- Preview mode will automatically exit after 5 seconds of inactivity
- The frontend must call this endpoint every 4-5 seconds to keep preview mode active

#### Success Response

- **Code**: 200 OK
- **Content**: None

#### Error Response

- **Code**: 404 Not Found (if not in preview mode)
- **Content**: None

## Data Structures

### DisplayContent

```json
{
  "id": "string", // Optional - will be generated if omitted
  "duration": number, // OPTIONAL - Display duration in seconds (0 = indefinite)
  "repeat_count": number, // OPTIONAL - Number of times to repeat (0 = indefinite)
  "border_effect": { // Optional border effect
    "None": null, // or
    "Rainbow": null, // or
    "Pulse": {"colors": [[R, G, B], ...]}, // or
    "Sparkle": {"colors": [[R, G, B], ...]}, // or
    "Gradient": {"colors": [[R, G, B], ...]}
  },
  "content": {
    "type": "Text", // Content type (currently only Text supported)
    "data": { ... } // Content-specific data structure
  }
}
```

**Important Note on Duration and Repeat Count:**
- Exactly ONE of `duration` or `repeat_count` must be specified
- Both fields cannot be specified simultaneously
- Neither field can be omitted entirely
- When using Text content with `scroll: true`, you MUST use `repeat_count` instead of `duration`

**Field Details:**
- `duration`: Time in seconds to display the content before transitioning to the next item. Use 0 for indefinite duration.
- `repeat_count`: Number of scroll cycles to complete before transitioning to the next item. Use 0 for infinite repetition.

### ContentType

Available content types:
- `Text` - Text content (currently the only supported type)
- Future types: `Image`, `Clock`, `Animation`, etc.

### ContentData

The structure of the `data` field depends on the `type` value:

For `Text` type, see [TextContent](#textcontent).

### TextContent

```json
{
  "text": "string", // Text content to display
  "scroll": boolean, // Whether to scroll the text
  "color": [R, G, B], // RGB color as a tuple of integers (0-255)
  "speed": number, // Scroll speed in pixels per second
  "text_segments": [ // Optional text formatting segments
    { ... } // See TextSegment structure
  ]
}
```

### BorderEffect

Border effects add visual effects around the displayed content:

- `None`: No border effect
- `Rainbow`: Colorful rainbow animation around the border
- `Pulse`: Border pulses with the specified colors
- `Sparkle`: Sparkling effect with the specified colors
- `Gradient`: Gradient animation with the specified colors

### TextSegment

Text segments allow different parts of the text to have different colors and formatting:

```json
{
  "start": 0, // Start index in the text
  "end": 5, // End index in the text (exclusive)
  "color": [255, 0, 0], // Optional RGB color for this segment
  "formatting": { // Optional formatting for this segment
    "bold": false,
    "underline": false,
    "strikethrough": false
  }
}
```

### TextFormatting

```json
{
  "bold": boolean, // Whether to make the text bold
  "underline": boolean, // Whether to underline the text
  "strikethrough": boolean // Whether to strike through the text
}
```