use std::collections::HashMap;
use std::sync::Arc;
use serde::de::Unexpected::Option;
use tokio::sync::{broadcast, Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use crate::sync_notes::{SyncRequest, SyncResponse, NoteChange};
use tonic::{Status, Streaming};
use crate::services::handlers::{handle_connection, handle_note_change};
use crate::sync_notes::sync_request::Action;

type NoteChannel = broadcast::Sender<NoteChange>;
pub struct NoteChannels(pub(crate) Arc<Mutex<HashMap<String, NoteChannel>>>);
impl NoteChannels {
    pub(crate) fn new() -> NoteChannels {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub(crate) async fn get_or_create_channel(&self, note_id: &str) -> broadcast::Receiver<NoteChange> {
        let mut channels = self.0.lock().await;
        let channel = channels.entry(note_id.to_string())
            .or_insert_with(|| broadcast::channel(32).0); // Create a new channel if one doesn't exist
        channel.subscribe()
    }

    pub(crate) fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct SyncManager {
    // HashMap to track broadcast channels per note ID
    channels: NoteChannels,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            channels: NoteChannels::new(),
        }
    }

    /// Ensure a broadcast channel exists for the given note ID and return a receiver.


    pub async fn start_sync(
        &self,
        mut stream: Streaming<SyncRequest>,
    ) -> Result<ReceiverStream<Result<SyncResponse, Status>>, Status> {
        // Channel to send responses back to the client
        let (client_tx, client_rx) = mpsc::channel(32);

        // Spawn task for actions received by streaming
        let channels = self.channels.clone();
        tokio::spawn(async move {
            while let Some(request) = stream.message().await.unwrap_or_else(|_| None) {
                if let Some(sync_action) = request.action {
                    match sync_action {
                        Action::NoteChange(note_change) => {
                            let current_note_id = note_change.note_id.clone();
                            let _ = handle_note_change(note_change, channels.clone()).await;


                        }
                        Action::Connection(connection) => {
                            let mut rx = channels.get_or_create_channel(&*request.note_id).await;
                            let client_txx = client_tx.clone();
                            tokio::spawn(async move {
                                while let Ok(note_change) = rx.recv().await {
                                    let response = SyncResponse { note_change: Some(note_change) };
                                    let _ = client_txx.send(Ok(response)).await;
                                }
                            });

                            // TODO move^^ to method
                            handle_connection(connection);
                        }
                    }
                } else {
                    eprintln!("Received an empty SyncRequest");
                }
            }
        });

        Ok(ReceiverStream::new(client_rx))
    }

    /// TODO, call this every few minutes in a separate task
    pub async fn cleanup_channels(&self) {
        let mut channels = self.channels.0.lock().await;
        channels.retain(|_, sender| sender.receiver_count() > 0);
    }
}
