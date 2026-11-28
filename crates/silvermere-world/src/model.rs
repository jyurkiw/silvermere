//! The builder format: what the world editor reads and writes, one TOML file per area.
//!
//! Records the content adds are named, not numbered (`stable-yard`); the compiler gives them
//! numbers. Stock rooms are named by number, as `/room` prints them (`"1/2140"`).

use serde::{Deserialize, Serialize};

/// One area file (`areas/newhaven-stable.toml`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Area {
    pub area: AreaHead,
    /// `[[room]]`: the rooms the area adds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub room: Vec<Room>,
    /// `[[exit]]`: exits between its rooms, and from stock rooms into them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exit: Vec<ExitLine>,
}

/// `[area]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreaHead {
    /// What the editor calls it.
    pub name: String,
    /// The map its rooms are on (exits never leave a map).
    pub map: u32,
}

/// A room the content adds.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    /// Its name, unique among the content's rooms: lower case, no `/`.
    pub name: String,
    /// The stock room it copies what it doesn't say (`"1/2140"`).
    pub like: String,
    /// The editor's position, x east, y up, z south. Dropped by the compiler.
    #[serde(default)]
    pub at: [i32; 3],
    /// The name players see, at most 52 characters.
    pub title: String,
    /// The description: at most 7 lines of at most 70 characters.
    #[serde(default)]
    pub desc: String,
    /// `Normal`, `Shop`, ... (the copied room's when left out).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Random monsters it holds at most (0: none).
    #[serde(default)]
    pub max_monsters: u8,
    /// The stock monster always here (an NPC), by number. None when left out: a new room
    /// never takes the copied room's NPC, shop or script.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc: Option<i32>,
    /// The monster group its random monsters come from, a `MonsterType` code (`Group 1` is
    /// 6); the copied room's when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mon_group: Option<i16>,
    /// The levels (`genindex`) of the group's monsters it makes, 0 for no limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_level: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_level: Option<i16>,
    /// One stock monster it makes instead of the group's, by number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gen_monster: Option<i32>,
}

/// An exit, both ways unless `one_way`: the compiler writes the reverse side too.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExitLine {
    /// A room by name, or a stock room as `"1/2140"`.
    pub from: String,
    /// `north`, `ne`, `up`, ...
    pub dir: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub one_way: bool,
}

/// The claims sidecar (`builder.claims.toml`): which number each name was given. The next
/// compile keeps them where it can, so a room keeps its number.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claims {
    /// Room name to `"map/room"`.
    #[serde(default)]
    pub rooms: std::collections::BTreeMap<String, String>,
}

impl Area {
    /// The area file's text.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn from_toml(text: &str) -> Result<Area, toml::de::Error> {
        toml::from_str(text)
    }
}

impl Claims {
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        Ok(format!(
            "# Claims of builder.toml: the number each name was compiled to. Written by\n\
             # silvermere-world {}; don't edit.\n{}",
            env!("CARGO_PKG_VERSION"),
            toml::to_string(self)?
        ))
    }

    pub fn from_toml(text: &str) -> Result<Claims, toml::de::Error> {
        toml::from_str(text)
    }
}
