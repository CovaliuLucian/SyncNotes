use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteChange {
    pub note_id: String,
    pub user_id: String,
    pub change_type: String, // "create", "update", or "delete"
    // Other fields like title, content, timestamps, etc.
}
