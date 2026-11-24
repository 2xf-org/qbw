//! How the company keeps its books.

mod aging_basis;
mod company_info;
mod fact;
mod facts;
mod preference;
mod preferences;
mod product_version;
mod user;
mod users;

pub use aging_basis::AgingBasis;
pub use company_info::CompanyInfo;
pub use fact::Fact;
pub use facts::Facts;
pub use preference::Preference;
pub use preferences::Preferences;
pub use product_version::ProductVersion;
pub use user::User;
pub use users::Users;

pub(crate) use company_info::read as information;
pub(crate) use facts::read as facts;
pub(crate) use preferences::read;
pub(crate) use users::read as users;
