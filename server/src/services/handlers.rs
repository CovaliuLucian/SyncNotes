use crate::sync_notes::{NoteChange, NoteConnect};
use tonic::Status;

/// Handle a note change action
pub async fn handle_note_change(note_change: NoteChange, channels: crate::services::sync::NoteChannels) -> Result<(), Status> {
    let note_id = note_change.note_id.clone();
    
    // Persist data
    // TODO
    println!("Received NoteChange {:?}", note_change);

    // Broadcast to others
    if let Some(channel) = channels.0.lock().await.get(&note_id) {
        channel.send(note_change).map_err(|_| Status::internal("Failed to broadcast note change"))?;
    }

    Ok(())
}

/// Handle a client ID action
pub fn handle_connection(connection: NoteConnect) {
    println!("Client connected: {:?}", connection);
}
