use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{RunExperimentOptions, ServiceResult};

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Job type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    ExperimentRun,
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub progress: Option<JobProgress>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

/// Job progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub current: usize,
    pub total: usize,
    pub message: Option<String>,
}

/// Internal job state
struct Job {
    info: JobInfo,
    cancel_flag: Arc<RwLock<bool>>,
}

/// Job manager for tracking async jobs
pub struct JobManager {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new job for running an experiment
    pub fn create_experiment_run_job(
        &self,
        options: &RunExperimentOptions,
    ) -> ServiceResult<String> {
        let id = Uuid::new_v4().to_string();

        let job = Job {
            info: JobInfo {
                id: id.clone(),
                job_type: JobType::ExperimentRun,
                status: JobStatus::Pending,
                created_at: chrono::Utc::now(),
                started_at: None,
                completed_at: None,
                progress: None,
                error: None,
                metadata: serde_json::json!({
                    "experiment_name": options.experiment_name,
                    "tests": options.tests,
                    "short_circuit": options.short_circuit,
                    "parallel": options.parallel,
                }),
            },
            cancel_flag: Arc::new(RwLock::new(false)),
        };

        let mut jobs = self.jobs.write().unwrap();
        jobs.insert(id.clone(), job);

        Ok(id)
    }

    /// Get job information by ID
    pub fn get_job(&self, id: &str) -> ServiceResult<JobInfo> {
        let jobs = self.jobs.read().unwrap();
        jobs.get(id)
            .map(|j| j.info.clone())
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))
    }

    /// List all jobs
    pub fn list_jobs(&self) -> ServiceResult<Vec<JobInfo>> {
        let jobs = self.jobs.read().unwrap();
        let job_list: Vec<JobInfo> = jobs.values().map(|j| j.info.clone()).collect();
        Ok(job_list)
    }

    /// List jobs with optional status filter
    pub fn list_jobs_by_status(&self, status: Option<JobStatus>) -> ServiceResult<Vec<JobInfo>> {
        let jobs = self.jobs.read().unwrap();
        let job_list: Vec<JobInfo> = jobs
            .values()
            .filter(|j| status.is_none() || j.info.status == *status.as_ref().unwrap())
            .map(|j| j.info.clone())
            .collect();
        Ok(job_list)
    }

    /// Update job status
    pub fn update_job_status(&self, id: &str, status: JobStatus) -> ServiceResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        job.info.status = status.clone();

        match status {
            JobStatus::Running => {
                job.info.started_at = Some(chrono::Utc::now());
            }
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                job.info.completed_at = Some(chrono::Utc::now());
            }
            _ => {}
        }

        Ok(())
    }

    /// Update job progress
    pub fn update_job_progress(&self, id: &str, progress: JobProgress) -> ServiceResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        job.info.progress = Some(progress);

        Ok(())
    }

    /// Set job error
    pub fn set_job_error(&self, id: &str, error: String) -> ServiceResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        job.info.status = JobStatus::Failed;
        job.info.completed_at = Some(chrono::Utc::now());
        job.info.error = Some(error);

        Ok(())
    }

    /// Cancel a job
    pub fn cancel_job(&self, id: &str) -> ServiceResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        // Only pending or running jobs can be cancelled
        if job.info.status != JobStatus::Pending && job.info.status != JobStatus::Running {
            bail!(
                "Job {} is not pending or running (status: {:?})",
                id,
                job.info.status
            );
        }

        // Set cancel flag
        let mut cancel_flag = job.cancel_flag.write().unwrap();
        *cancel_flag = true;

        job.info.status = JobStatus::Cancelled;
        job.info.completed_at = Some(chrono::Utc::now());

        Ok(())
    }

    /// Check if a job is cancelled
    pub fn is_job_cancelled(&self, id: &str) -> ServiceResult<bool> {
        let jobs = self.jobs.read().unwrap();
        let job = jobs
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        let cancel_flag = job.cancel_flag.read().unwrap();
        Ok(*cancel_flag)
    }

    /// Get the cancel flag for a job (for use in async execution)
    pub fn get_cancel_flag(&self, id: &str) -> ServiceResult<Arc<RwLock<bool>>> {
        let jobs = self.jobs.read().unwrap();
        let job = jobs
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

        Ok(Arc::clone(&job.cancel_flag))
    }

    /// Delete completed/failed/cancelled jobs older than the specified duration
    pub fn cleanup_old_jobs(&self, max_age: chrono::Duration) -> ServiceResult<usize> {
        let mut jobs = self.jobs.write().unwrap();
        let now = chrono::Utc::now();
        let mut removed = 0;

        jobs.retain(|_, job| {
            // Keep pending and running jobs
            if job.info.status == JobStatus::Pending || job.info.status == JobStatus::Running {
                return true;
            }

            // Check age of completed jobs
            if let Some(completed_at) = job.info.completed_at {
                if now.signed_duration_since(completed_at) > max_age {
                    removed += 1;
                    return false;
                }
            }

            true
        });

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_lifecycle() {
        let manager = JobManager::new();

        // Create a job
        let options = RunExperimentOptions {
            experiment_name: "test".to_string(),
            tests: vec!["test1".to_string()],
            short_circuit: false,
            parallel: false,
            params: vec![],
        };

        let job_id = manager.create_experiment_run_job(&options).unwrap();

        // Check initial status
        let job = manager.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Pending);

        // Update to running
        manager
            .update_job_status(&job_id, JobStatus::Running)
            .unwrap();
        let job = manager.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());

        // Update progress
        manager
            .update_job_progress(
                &job_id,
                JobProgress {
                    current: 1,
                    total: 10,
                    message: Some("Running test 1".to_string()),
                },
            )
            .unwrap();
        let job = manager.get_job(&job_id).unwrap();
        assert!(job.progress.is_some());
        assert_eq!(job.progress.unwrap().current, 1);

        // Complete the job
        manager
            .update_job_status(&job_id, JobStatus::Completed)
            .unwrap();
        let job = manager.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_job_cancellation() {
        let manager = JobManager::new();

        let options = RunExperimentOptions {
            experiment_name: "test".to_string(),
            tests: vec!["test1".to_string()],
            short_circuit: false,
            parallel: false,
            params: vec![],
        };

        let job_id = manager.create_experiment_run_job(&options).unwrap();
        manager
            .update_job_status(&job_id, JobStatus::Running)
            .unwrap();

        // Cancel the job
        manager.cancel_job(&job_id).unwrap();

        let job = manager.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(manager.is_job_cancelled(&job_id).unwrap());
    }
}
