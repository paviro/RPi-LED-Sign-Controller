use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::{
    extract::State,
    response::{sse::Event, Sse},
};
use futures::stream::{self, Stream};
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::StreamExt as _;
use crate::web::api::CombinedState;
use crate::models::settings::BrightnessSettings;
use crate::models::playlist::PlayListItem;
use serde::{Serialize, Deserialize};

// Define event types for editor lock
#[derive(Clone, Serialize, Deserialize)]
pub struct EditorLockEvent {
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_by: Option<String>,
}

// Define event types for playlist updates
#[derive(Clone, Serialize, Deserialize)]
pub struct PlaylistUpdateEvent {
    pub items: Vec<PlayListItem>,
    pub action: PlaylistAction,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PlaylistAction {
    Add,
    Update,
    Delete,
    Reorder,
}

// Singleton for managing all event types
pub struct EventState {
    brightness_tx: Sender<BrightnessSettings>,
    editor_lock_tx: Sender<EditorLockEvent>,
    playlist_tx: Sender<PlaylistUpdateEvent>,
}

impl EventState {
    pub fn new() -> Arc<Mutex<Self>> {
        let (brightness_tx, _) = broadcast::channel(100);
        let (editor_lock_tx, _) = broadcast::channel(100);
        let (playlist_tx, _) = broadcast::channel(100);
        
        Arc::new(Mutex::new(Self {
            brightness_tx,
            editor_lock_tx,
            playlist_tx,
        }))
    }
    
    pub fn get_brightness_sender(&self) -> Sender<BrightnessSettings> {
        self.brightness_tx.clone()
    }
    
    pub fn broadcast_brightness(&self, brightness: BrightnessSettings) {
        let _ = self.brightness_tx.send(brightness);
    }
    
    pub fn get_editor_lock_sender(&self) -> Sender<EditorLockEvent> {
        self.editor_lock_tx.clone()
    }
    
    pub fn broadcast_editor_lock(&self, is_locked: bool, locked_by: Option<String>) {
        let event = EditorLockEvent {
            locked: is_locked,
            locked_by,
        };
        let _ = self.editor_lock_tx.send(event);
    }
    
    pub fn get_playlist_sender(&self) -> Sender<PlaylistUpdateEvent> {
        self.playlist_tx.clone()
    }
    
    pub fn broadcast_playlist_update(&self, items: Vec<PlayListItem>, action: PlaylistAction) {
        let event = PlaylistUpdateEvent {
            items,
            action,
        };
        let _ = self.playlist_tx.send(event);
    }
}

pub type SharedEventState = Arc<Mutex<EventState>>;

// Handler for brightness SSE events
pub async fn brightness_events(
    State(combined_state): State<CombinedState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let brightness_rx = {
        let (_, event_state) = &combined_state;
        let event_state = event_state.lock().unwrap();
        event_state.get_brightness_sender().subscribe()
    };
    
    let stream = stream::unfold(brightness_rx, |mut rx| async move {
        match rx.recv().await {
            Ok(brightness) => {
                let payload = serde_json::to_string(&brightness).unwrap();
                let event = Event::default().data(payload);
                Some((Ok(event), rx))
            }
            Err(_) => {
                // Keep connection alive with a comment
                let event = Event::default().event("ping").data("");
                Some((Ok(event), rx))
            }
        }
    });
    
    // Add keepalive logic
    let keepalive = stream::repeat_with(|| {
        Event::default().event("ping").data("")
    })
    .map(Ok)
    .throttle(Duration::from_secs(30));
    
    Sse::new(stream.merge(keepalive))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text")
        )
}

// Handler for editor lock SSE events
pub async fn editor_lock_events(
    State(combined_state): State<CombinedState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let lock_rx = {
        let (_, event_state) = &combined_state;
        let event_state = event_state.lock().unwrap();
        event_state.get_editor_lock_sender().subscribe()
    };
    
    let stream = stream::unfold(lock_rx, |mut rx| async move {
        match rx.recv().await {
            Ok(lock_event) => {
                let payload = serde_json::to_string(&lock_event).unwrap();
                let event = Event::default().data(payload);
                Some((Ok(event), rx))
            }
            Err(_) => {
                // Keep connection alive with a comment
                let event = Event::default().event("ping").data("");
                Some((Ok(event), rx))
            }
        }
    });
    
    // Add keepalive logic
    let keepalive = stream::repeat_with(|| {
        Event::default().event("ping").data("")
    })
    .map(Ok)
    .throttle(Duration::from_secs(30));
    
    Sse::new(stream.merge(keepalive))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text")
        )
}

// Handler for playlist update SSE events
pub async fn playlist_events(
    State(combined_state): State<CombinedState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let playlist_rx = {
        let (_, event_state) = &combined_state;
        let event_state = event_state.lock().unwrap();
        event_state.get_playlist_sender().subscribe()
    };
    
    let stream = stream::unfold(playlist_rx, |mut rx| async move {
        match rx.recv().await {
            Ok(playlist_event) => {
                let payload = serde_json::to_string(&playlist_event).unwrap();
                let event = Event::default().data(payload);
                Some((Ok(event), rx))
            }
            Err(_) => {
                // Keep connection alive with a comment
                let event = Event::default().event("ping").data("");
                Some((Ok(event), rx))
            }
        }
    });
    
    // Add keepalive logic
    let keepalive = stream::repeat_with(|| {
        Event::default().event("ping").data("")
    })
    .map(Ok)
    .throttle(Duration::from_secs(30));
    
    Sse::new(stream.merge(keepalive))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text")
        )
}