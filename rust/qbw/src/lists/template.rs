//! Templates of forms.

use sa17::Source;

use crate::codes::TemplateForm;
use crate::error::Result;
use crate::ids::TemplateId;
use crate::list::{List, ListEntry, sealed};
use crate::lists::reading;
use crate::origin::Origin;
use crate::tables::lists::TEMPLATES;

/// What a form is printed with.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Template {
    /// What the template is known by.
    pub id: TemplateId,
    /// Its name.
    pub name: String,
    /// The form it is for.
    pub form: TemplateForm,
    /// Whether it can be used in new transactions.
    pub is_active: bool,
    /// The stored row.
    pub origin: Origin,
}

impl sealed::Sealed for Template {}

impl ListEntry for Template {
    type Id = TemplateId;

    fn id(&self) -> TemplateId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn origin(&self) -> Origin {
        self.origin
    }
}

/// Reads the templates.
///
/// # Errors
///
/// Returns an error when their table lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<List<Template>> {
    reading::by_name(source, &TEMPLATES, |origin, row| {
        (!row.is_deleted).then(|| Template {
            id: row.id,
            name: row.name.unwrap_or_default(),
            form: row.form,
            is_active: !row.is_hidden,
            origin,
        })
    })
}
