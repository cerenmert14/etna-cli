use std::sync::{Arc, RwLock};

use crate::manager::Manager;
use crate::service::job::JobManager;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<RwLock<Manager>>,
    pub job_manager: Arc<JobManager>,
}

impl AppState {
    pub fn new(manager: Manager, job_manager: JobManager) -> Self {
        Self {
            manager: Arc::new(RwLock::new(manager)),
            job_manager: Arc::new(job_manager),
        }
    }
}
