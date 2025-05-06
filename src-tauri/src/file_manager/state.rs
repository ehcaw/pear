use crate::file_manager::neo4j::NeoDB;
use anyhow::Error;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;

pub struct TrackerHandle {
    pub repository_id: String,
    pub cancel_sender: oneshot::Sender<()>,
}

#[derive(Default)]
pub struct AppState {
    pub neo4j_updater: Mutex<Option<NeoDB>>,
    pub active_trackers: Mutex<HashMap<String, TrackerHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            neo4j_updater: Mutex::new(None),
            active_trackers: Mutex::new(HashMap::new()),
        }
    }

    pub async fn initialize(
        &self,
        uri: &str,
        user: &str,
        password: &str,
        repository_id: String,
        owner_id: String,
    ) -> Result<(), Error> {
        let updater = NeoDB::new(uri, user, password, repository_id, owner_id).await?;
        *self.neo4j_updater.lock().unwrap() = Some(updater);
        Ok(())
    }

    pub fn start_tracking(
        &self,
        repository_path: String,
        repository_id: String,
    ) -> Result<(), Error> {
        let (cancel_sender, _cancel_receiver) = oneshot::channel();

        let handle = TrackerHandle {
            repository_id: repository_id.clone(),
            cancel_sender,
        };

        self.active_trackers
            .lock()
            .unwrap()
            .insert(repository_path, handle);
        Ok(())
    }

    pub fn stop_tracking(&self, repository_path: &str) {
        if let Some(handle) = self.active_trackers.lock().unwrap().remove(repository_path) {
            let _ = handle.cancel_sender.send(());
        }
    }
}
