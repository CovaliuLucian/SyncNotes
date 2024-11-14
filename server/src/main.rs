mod services;

use crate::services::NotesService;
use sync_notes::sync_notes_server::{SyncNotes, SyncNotesServer};
use tonic::transport::Server;

pub mod sync_notes {
    tonic::include_proto!("sync_notes");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = NotesService::default();

    Server::builder()
        .add_service(SyncNotesServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}