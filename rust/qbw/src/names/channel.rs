//! Phone numbers, e-mail addresses and the like.

use std::collections::HashMap;

use sa17::Source;

use crate::codes::ChannelKind;
use crate::damage::Damage;
use crate::error::Result;
use crate::ids::NameId;
use crate::origin::Origin;
use crate::tables::contacts::{CHANNEL_LABELS, CHANNELS};

/// A phone number, an e-mail address or another way to reach a name, with what
/// the company calls it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Channel {
    /// The name it reaches.
    pub name: NameId,
    /// What it is.
    pub kind: ChannelKind,
    /// What the company calls it, such as `Main Phone`.
    pub label: Option<String>,
    /// The number or the address.
    pub value: String,
    /// Its place among the eight that QuickBooks shows, from 1 on.
    pub position: Option<i32>,
    /// The stored row.
    pub origin: Origin,
}

/// Reads the channels of every name, each name's in their order.
///
/// # Errors
///
/// Returns an error when a table of them lacks a column.
pub(super) fn read(source: &dyn Source, damage: &mut Vec<Damage>) -> Result<Vec<Channel>> {
    let labels = match CHANNEL_LABELS.find(source)? {
        Some(table) => table
            .scan(damage)
            .filter_map(|(_, row)| Some((row.kind, row.text?)))
            .collect(),
        None => HashMap::new(),
    };
    let Some(table) = CHANNELS.find(source)? else {
        return Ok(Vec::new());
    };
    let mut channels: Vec<Channel> = table
        .scan(damage)
        .filter(|(_, row)| !row.is_deleted)
        .filter_map(|(origin, row)| {
            Some(Channel {
                name: row.name,
                kind: row.kind,
                label: labels.get(&row.kind).cloned(),
                value: row.value?,
                position: row.position,
                origin,
            })
        })
        .collect();
    channels.sort_by_key(|channel| (channel.name, channel.position));
    Ok(channels)
}
