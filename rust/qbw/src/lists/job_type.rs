//! Types of jobs.

use sa17::Source;

use crate::error::Result;
use crate::ids::JobTypeId;
use crate::list::List;
use crate::lists::reading::Place;
use crate::lists::{Label, reading};
use crate::tables::lists::{JOB_TYPES, JobTypeRow};

/// A type of jobs.
pub type JobType = Label<JobTypeId>;

/// Reads the job types.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<JobType>> {
    let place = |row: &JobTypeRow| Place::of(row.id.number(), row.next);
    reading::in_stored_order(source, &JOB_TYPES, place, |origin, row| {
        (!row.is_deleted).then(|| Label {
            id: row.id,
            name: row.name.unwrap_or_default(),
            parent: row.parent,
            is_active: !row.is_hidden,
            origin,
        })
    })
}
