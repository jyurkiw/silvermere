//! Fork: pets (`pets`, design/warden.md): a character's companion and, later, its
//! other pets. Keyed by account and source; pet kinds are content, not a table.

use silvermere_types::fork::Pet;
use silvermere_types::ids::AccountId;

use crate::Result;
use crate::pool::Db;
use crate::repo::{Args, Get, upsert_sql};

const COLS: &[&str] = &[
    "account_id",
    "source",
    "kind",
    "name",
    "summoned",
    "immortal",
    "dead",
];

fn flag(b: bool) -> i64 {
    i64::from(b)
}

/// Pets.
#[derive(Debug, Clone, Copy)]
pub struct Pets<'a> {
    db: &'a Db,
}

impl<'a> Pets<'a> {
    pub(crate) fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Every pet of `account`.
    pub async fn of_account(&self, account: AccountId) -> Result<Vec<Pet>> {
        let mut a = Args::new();
        a.add(account.0);
        let rows = a
            .query(
                "SELECT source, kind, name, summoned, immortal, dead FROM pets \
                 WHERE account_id = $1 ORDER BY id"
                    .to_string(),
            )
            .fetch_all(&self.db.pool())
            .await?;
        rows.iter()
            .map(|r| {
                let g = Get::new(r, "pets");
                Ok(Pet {
                    account,
                    source: g.text("source")?,
                    kind: g.int("kind")?,
                    name: g.text("name")?,
                    summoned: g.i64("summoned")? != 0,
                    immortal: g.i64("immortal")? != 0,
                    dead: g.i64("dead")? != 0,
                })
            })
            .collect()
    }

    /// Writes `pet`, replacing the account's pet of the same source.
    pub async fn put(&self, pet: &Pet) -> Result<()> {
        let mut a = Args::new();
        a.add(pet.account.0)
            .add(&pet.source)
            .add(i64::from(pet.kind))
            .add(&pet.name)
            .add(flag(pet.summoned))
            .add(flag(pet.immortal))
            .add(flag(pet.dead));
        a.query(upsert_sql("pets", COLS, &["account_id", "source"]))
            .execute(&self.db.pool())
            .await?;
        Ok(())
    }
}
