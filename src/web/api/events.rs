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

// Singleton for managing SSE events
pub struct EventState {
    brightness_tx: Sender<BrightnessSettings>,
}

impl EventState {
    pub fn new() -> Arc<Mutex<Self>> {
        let (brightness_tx, _) = broadcast::channel(100);
        
        Arc::new(Mutex::new(Self {
            brightness_tx,
        }))
    }
    
    pub fn get_brightness_sender(&self) -> Sender<BrightnessSettings> {
        self.brightness_tx.clone()
    }
    
    pub fn broadcast_brightness(&self, brightness: BrightnessSettings) {
        let _ = self.brightness_tx.send(brightness);
    }
}

pub type SharedEventState = Arc<Mutex<EventState>>;

// Handler for brightness SSE events
pub async fn brightness_events(
    State(combined_state): State<CombinedState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let brightness_rx = {
        let (_, sse_state) = &combined_state;
        let sse_state = sse_state.lock().unwrap();
        sse_state.get_brightness_sender().subscribe()
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
    
    // Add keepalive logic - fixed by directly returning the Event
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