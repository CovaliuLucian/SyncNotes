use super::sync::SyncManager;
use crate::sync_notes::sync_notes_server::SyncNotes;
use crate::sync_notes::{FooReply, FooRequest, SyncRequest, SyncResponse};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

pub struct NotesService {
    sync_manager: SyncManager,
}

impl Default for NotesService {
    fn default() -> Self {
        Self {
            sync_manager: SyncManager::new(),
        }
    }
}

#[tonic::async_trait]
impl SyncNotes for NotesService {
    async fn foo(&self, request: Request<FooRequest>) -> Result<Response<FooReply>, Status> {

        println!("Got a foo request: {:?}", request);
        let reply = FooReply {
            result: format!("Got {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    type SyncNotesStream = ReceiverStream<Result<SyncResponse, Status>>;

    async fn sync_notes(
        &self,
        request: Request<Streaming<SyncRequest>>,
    ) -> Result<Response<Self::SyncNotesStream>, Status> {
        let sync_stream = self.sync_manager.start_sync(request.into_inner()).await?;
        Ok(Response::new(sync_stream))
    }
}