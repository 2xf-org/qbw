//! The people who work on the books.

use std::collections::BTreeMap;

use sa17::Source;

use crate::damage::Damage;
use crate::error::Result;
use crate::ids::UserId;
use crate::left_out::LeftOut;
use crate::settings::User;
use crate::tables::information::{SAVES, SaveRow};
use crate::unix_time::UnixTime;

/// The people who work on the books in QuickBooks, as the audit trail names
/// them.
#[derive(Clone, Debug)]
pub struct Users {
    /// Ordered by id.
    users: Vec<User>,
    damage: Vec<Damage>,
}

impl Users {
    /// The user with `id`.
    #[must_use]
    pub fn get(&self, id: UserId) -> Option<&User> {
        self.iter().find(|user| user.id == id)
    }

    /// The users, ordered by id.
    pub fn iter(&self) -> std::slice::Iter<'_, User> {
        self.users.iter()
    }

    /// The number of users.
    #[must_use]
    pub fn len(&self) -> usize {
        self.users.len()
    }

    /// Whether there are no users.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// How much could not be read.
    #[must_use]
    pub fn left_out(&self) -> LeftOut {
        LeftOut::of(&self.damage)
    }

    /// What could not be read.
    #[must_use]
    pub fn damage(&self) -> &[Damage] {
        &self.damage
    }
}

impl<'users> IntoIterator for &'users Users {
    type Item = &'users User;
    type IntoIter = std::slice::Iter<'users, User>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Reads the users from the audit trail, which a file may lack.
///
/// # Errors
///
/// Returns an error when the table of the audit trail lacks a column.
pub(crate) fn read(source: &dyn Source) -> Result<Users> {
    let mut damage = Vec::new();
    let mut latest = LatestSaves::default();
    if let Some(table) = SAVES.find(source)? {
        for (_, row) in table.scan(&mut damage) {
            latest.add(row);
        }
    }
    let users = latest.0.into_iter();
    let users = users.map(|(id, saves)| User {
        id,
        names: latest_first(saves),
    });
    Ok(Users {
        users: users.collect(),
        damage,
    })
}

/// The names of each user, with the time that each was last used.
#[derive(Default)]
struct LatestSaves(BTreeMap<UserId, BTreeMap<String, Option<UnixTime>>>);

impl LatestSaves {
    /// Adds who saved or deleted a transaction.
    fn add(&mut self, row: SaveRow) {
        self.add_name(row.user, row.user_name, row.time);
        self.add_name(row.deleting_user, row.deleting_user_name, row.delete_time);
    }

    fn add_name(&mut self, user: Option<UserId>, name: Option<String>, time: Option<UnixTime>) {
        let (Some(user), Some(name)) = (user, name) else {
            return;
        };
        let names = self.0.entry(user).or_default();
        let latest = names.entry(name).or_default();
        *latest = time.max(*latest);
    }
}

fn latest_first(saves: BTreeMap<String, Option<UnixTime>>) -> Vec<String> {
    let mut saves: Vec<(String, Option<UnixTime>)> = saves.into_iter().collect();
    saves.sort_by(|(_, left), (_, right)| right.cmp(left));
    let names = saves.into_iter().map(|(name, _)| name);
    names.collect()
}
