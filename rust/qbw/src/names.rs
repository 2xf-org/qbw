//! Customers and jobs, vendors, employees and other names, and how to reach
//! them.

mod channel;
mod contact;
mod customer;
mod details;
mod directory;
mod employee;
mod job;
mod key_index;
mod name;
mod name_kind;
mod other_name;
mod ship_to;
mod unsorted;
mod vendor;

pub use channel::Channel;
pub use contact::Contact;
pub use customer::Customer;
pub use directory::Names;
pub use employee::Employee;
pub use job::Job;
pub use name::Name;
pub use name_kind::NameKind;
pub use other_name::OtherName;
pub use ship_to::ShipTo;
pub use vendor::Vendor;

pub use crate::codes::{ChannelKind, ContactKind, JobStatus};

pub(crate) use contact::{contact, contact_index};
pub(crate) use directory::read;
pub(crate) use key_index::KeyIndex;
pub(crate) use ship_to::{ship_to, ship_to_index};
