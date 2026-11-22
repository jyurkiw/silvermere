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

/// `content/pets.toml`: kinds of pet (design/warden.md).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetsFile {
    /// `[[pet_kind]]`: one kind.
    #[serde(default)]
    pub pet_kind: Vec<PetKindEntry>,
}

/// One kind of pet, as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetKindEntry {
    /// The kind's number, the value of a character's `PetKind` ability.
    pub number: u16,
    /// Its name before it's given one: "hunting hound".
    pub name: String,
    /// The verbs of a hit and a miss: "bites", "snaps at".
    pub hit_verb: String,
    pub miss_verb: String,
    /// Blows a round, as energy per blow (a round gives 1000).
    pub energy_per_blow: u16,
    /// Damage as `"2-8"` at level 0, and what each level adds to each end, in hundredths.
    pub damage: String,
    pub per_level: [i16; 2],
    /// Added to the owner's attack value.
    #[serde(default)]
    pub accuracy: i16,
    /// Poison a hit leaves.
    #[serde(default)]
    pub poison: i16,
}

/// `content/world.toml`: new rooms, monsters, shops, items and text blocks, and changes to
/// stock rooms and items (design/warden.md: the Newhaven stable). A new record copies a
/// stock one (`like`) and overrides what it names.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldFile {
    #[serde(default)]
    pub room: Vec<RoomSpec>,
    #[serde(default)]
    pub room_change: Vec<RoomChange>,
    #[serde(default)]
    pub monster: Vec<MonsterSpec>,
    #[serde(default)]
    pub shop: Vec<ShopSpec>,
    #[serde(default)]
    pub item: Vec<ItemSpec>,
    #[serde(default)]
    pub item_change: Vec<ItemChange>,
    #[serde(default)]
    pub text: Vec<TextSpec>,
}

/// A new room.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomSpec {
    /// `"1/9000"`: map and room.
    pub room: String,
    /// The stock room it copies, `"1/2140"`.
    pub like: String,
    pub name: String,
    /// Up to 7 lines.
    pub desc: Vec<String>,
    /// Exits by direction (`north`, `east`, `ne`, `up`, ...) to rooms on the same map.
    #[serde(default)]
    pub exits: std::collections::BTreeMap<String, u32>,
    /// `Normal`, `Shop`, ... (the copied room's when left out).
    pub kind: Option<String>,
    /// The shop the room is (0 none).
    pub shop: Option<i32>,
    /// The monster always in the room (an NPC).
    pub specmon: Option<i32>,
    /// The text block of the room's commands (its script).
    pub script: Option<i32>,
    /// Random monsters the room may hold (the copied room's when left out; 0 for a safe room).
    pub max_monsters: Option<u8>,
}

/// Changes to a room already in the world: new exits, and the shop it is.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomChange {
    pub room: String,
    #[serde(default)]
    pub exits: std::collections::BTreeMap<String, u32>,
    /// The shop the room becomes (a fork shop that replaces a stock one).
    pub shop: Option<i32>,
}

/// A new known monster (an NPC).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonsterSpec {
    pub number: i32,
    pub like: i32,
    pub name: String,
    /// Up to 4 lines.
    pub desc: Vec<String>,
    /// Its talk block (`KEYWORDS:block` lines, linking to its greeting).
    pub talk: Option<i32>,
}

/// A new shop.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopSpec {
    pub number: i32,
    pub like: i32,
    pub name: String,
    /// What it sells: item numbers, each kept in stock.
    pub items: Vec<i32>,
}

/// A new item.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemSpec {
    pub number: i32,
    pub like: i32,
    pub name: String,
    /// Up to 6 lines.
    pub desc: Vec<String>,
    /// Its price, in the copied item's currency.
    #[serde(default)]
    pub cost: u16,
    #[serde(default)]
    pub abilities: Vec<AbilityEntry>,
    /// A weapon's type, `OneHandSharp`, ... (the copied item's when left out).
    pub weapon_type: Option<String>,
    /// Damage and speed (the copied item's when left out).
    pub mindmg: Option<i16>,
    pub maxdmg: Option<i16>,
    pub speed: Option<i16>,
    /// The item whose hit and miss verbs it uses (the copied item's when left out).
    pub verbs_like: Option<i32>,
}

/// Abilities added to an item already in the world (marking stock ranged weapons).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemChange {
    pub number: i32,
    pub add_abilities: Vec<AbilityEntry>,
}

/// A text block: a talk block, an answer, a room's script.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextSpec {
    pub number: i32,
    /// Lines; part 0 only, at most 1999 bytes.
    pub text: String,
    /// The block it links to (a talk block's greeting).
    #[serde(default)]
    pub link: i32,
}
