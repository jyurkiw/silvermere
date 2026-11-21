//! The shape of the content files, as written by hand.

use serde::Deserialize;

/// `content/classes.toml`: new classes and changes to stock ones.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassesFile {
    /// `[[class]]`: a class the content adds.
    #[serde(default)]
    pub class: Vec<NewClass>,
    /// `[[class_change]]`: a change to a class already in the world.
    #[serde(default)]
    pub class_change: Vec<ClassChange>,
}

/// A new class, in the help's terms rather than the record's.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewClass {
    /// The class number, which players type at character creation.
    pub number: u16,
    /// The class name.
    pub name: String,
    /// HP per level as the help shows it, `"5-9"`.
    pub hp: String,
    /// Experience as the help shows it, in percent (100 is a Warrior's).
    pub exp: i16,
    /// Magic as the help shows it, `"Mage-1"`, or `"None"`.
    pub magic: String,
    /// Fighting 1-5 as the help shows it (Warrior 4).
    pub fighting: i16,
    /// The widest weapon class, by its name (`"All"`, `"AnyOneHand"`, ...).
    pub weapons: String,
    /// The heaviest armour type, by its name (`"Chainmail"`, ...).
    pub armour: String,
    /// The class's abilities.
    #[serde(default)]
    pub abilities: Vec<AbilityEntry>,
    /// The level titles.
    pub titles: Option<Titles>,
}

/// One ability and its value.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbilityEntry {
    /// A stock ability by its name (`"Bash"`) or a fork ability (`"Spellstrike"`).
    pub name: String,
    /// The ability's value.
    #[serde(default)]
    pub value: i16,
}

/// A class's level titles, one per band.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Titles {
    /// The text block the titles are written to.
    pub block: i32,
    /// The ranks in level order; each holds until the next one's `from`.
    pub ranks: Vec<Rank>,
}

/// One rank of a title list.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rank {
    /// The first level the title is held at.
    pub from: u16,
    /// The title, or `"Male|Female"`.
    pub title: String,
}

/// A change to a class already in the world, named by its name.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassChange {
    /// The class's name as the world has it.
    pub name: String,
    /// Abilities to add, or to set when the class already has them.
    #[serde(default)]
    pub add_abilities: Vec<AbilityEntry>,
}

/// `content/help.toml`: HELP topics.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelpFile {
    /// `[[class_topic]]`: a class's topic in the stock class-topic box.
    #[serde(default)]
    pub class_topic: Vec<ClassTopic>,
    /// `[[topic]]`: any other topic, printed as written.
    #[serde(default)]
    pub topic: Vec<Topic>,
}

/// A class topic, drawn from the class's own record.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassTopic {
    /// The class, by name, one the content adds.
    pub class: String,
    /// The words players type, `"War Mages"`; the class name plus `s` when left out.
    pub words: Option<String>,
    /// The description, wrapped to the box.
    pub text: String,
    /// The abilities line, `"Spellstrike."`.
    pub abilities: String,
}

/// A plain topic.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    /// The words players type.
    pub words: String,
    /// The text, printed as written.
    pub text: String,
}
