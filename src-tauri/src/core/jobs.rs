use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::core::errors::VisionError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Cancelled,
    Failed(String), // We store the error message instead of the full error enum for easy serialization
}

/// A handle passed to engines/operations to check if they should stop work early.
#[derive(Clone, Default)]
pub struct CancellationHandle {
    is_cancelled: Arc<AtomicBool>,
}

impl CancellationHandle {
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Mark the job as cancelled.
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Relaxed);
    }

    /// Check if the job has been cancelled. Engines should call this periodically.
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::Relaxed)
    }

    /// Returns a CancellationError if the job has been cancelled, allowing early return via `?`.
    pub fn check(&self) -> Result<(), VisionError> {
        if self.is_cancelled() {
            Err(VisionError::CancellationError)
        } else {
            Ok(())
        }
    }
}

/// Represents an asynchronous job running in the background.
pub struct Job {
    pub id: String,
    pub state: JobState,
    pub cancellation_handle: CancellationHandle,
}

impl Job {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: JobState::Queued,
            cancellation_handle: CancellationHandle::new(),
        }
    }

    pub fn cancel(&mut self) {
        if self.state == JobState::Queued || self.state == JobState::Running {
            self.cancellation_handle.cancel();
            self.state = JobState::Cancelled;
        }
    }

    pub fn mark_running(&mut self) {
        if self.state == JobState::Queued {
            self.state = JobState::Running;
        }
    }

    pub fn mark_completed(&mut self) {
        self.state = JobState::Completed;
    }

    pub fn mark_failed(&mut self, err: VisionError) {
        self.state = JobState::Failed(err.to_string());
    }
}
