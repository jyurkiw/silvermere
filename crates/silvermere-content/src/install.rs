//! Writing the content into a database: at every server start, and in tests.
//!
//! New classes are written whole, by number, with their title blocks. Changes read the
//! class from the database, set only the abilities they name, and write it back. Both are
//! idempotent, so installing at every start is safe. A new class's number, or its title
//! block, taken by content that isn't this content's (a different class name) is refused
//! rather than overwritten.

use silvermere_db::Db;
use silvermere_types::enums::Ability;
use silvermere_types::ids::TextBlockNo;

use crate::{ClassEdit, Content};

/// Why the content couldn't be installed. The server doesn't start: a world without its
/// classes would load characters of a class that isn't there.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    /// The database refused.
    #[error(transparent)]
    Db(#[from] silvermere_db::DbError),
    /// Something else holds a number the content needs.
    #[error("content: {0}")]
    Taken(String),
    /// A change names a class the world doesn't have, or one with no room left.
    #[error("content: {0}")]
    Change(String),
}

/// What an install wrote.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InstallReport {
    /// New classes written.
    pub classes: usize,
    /// Title blocks written.
    pub titles: usize,
    /// Stock classes changed.
    pub changes: usize,
    /// World records written (rooms, monsters, shops, items, text blocks).
    pub world: usize,
}

/// Installs `content` into `db`.
pub async fn install(db: &Db, content: &Content) -> Result<InstallReport, InstallError> {
    let mut report = InstallReport::default();
    for class in &content.classes {
        let existing = db.classes().get(class.number).await?;
        let ours = match &existing {
            Some(e) if e.name != class.name => {
                return Err(InstallError::Taken(format!(
                    "class {} is `{}` in this world, not `{}`",
                    class.number.0, e.name, class.name
                )));
            }
            Some(_) => true,
            None => false,
        };
        if class.titles != TextBlockNo(0)
            && let Some(block) = content.titles.iter().find(|t| t.number == class.titles)
        {
            let taken = db.text_blocks().part(0, block.number).await?;
            // a block already there is ours only when the class already was
            if taken.is_some() && !ours {
                return Err(InstallError::Taken(format!(
                    "text block {} (the {} titles) is already in use",
                    block.number.0, class.name
                )));
            }
            db.text_blocks().put(block).await?;
            report.titles += 1;
        }
        db.classes().put(class).await?;
        report.classes += 1;
    }
    report.world = crate::world::install_world(db, &content.world).await?;
    for edit in &content.class_edits {
        apply_edit(db, edit).await?;
        report.changes += 1;
    }
    tracing::info!(
        classes = report.classes,
        titles = report.titles,
        changes = report.changes,
        world = report.world,
        "content installed"
    );
    Ok(report)
}

async fn apply_edit(db: &Db, edit: &ClassEdit) -> Result<(), InstallError> {
    let mut class = db
        .classes()
        .all()
        .await?
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(&edit.name))
        .ok_or_else(|| InstallError::Change(format!("no class `{}` to change", edit.name)))?;
    for &(ability, value) in &edit.abilities {
        if let Some(slot) = class.abilities.iter_mut().find(|s| s.0 == ability) {
            slot.1 = value;
        } else if let Some(slot) = class.abilities.iter_mut().find(|s| s.0 == Ability::None) {
            *slot = (ability, value);
        } else {
            return Err(InstallError::Change(format!(
                "class `{}` has no free ability slot",
                edit.name
            )));
        }
    }
    db.classes().put(&class).await?;
    Ok(())
}
