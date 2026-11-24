//! What is kept about a job.

use chrono::NaiveDate;

use crate::codes::JobStatus;
use crate::ids::JobTypeId;

/// How far a job is, and since when. In the files that this crate was checked
/// against, jobs have a status only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Job {
    /// How far it is.
    pub status: JobStatus,
    /// Its type.
    pub job_type: Option<JobTypeId>,
    /// What it is.
    pub description: Option<String>,
    /// The day it started.
    pub start: Option<NaiveDate>,
    /// The day it is expected to end.
    pub projected_end: Option<NaiveDate>,
    /// The day it ended.
    pub end: Option<NaiveDate>,
}
