//! The fork's own content: `content/*.toml`, built into the binary, checked, and installed
//! into the database at every server start.
//!
//! The world comes from the stock files at import time; an import overwrites stock records
//! and a new database has none of the fork's. Installing at every start puts the content
//! back after either, so nobody has to remember to. Installing is idempotent: the same
//! content gives the same rows. The stock files and the importer are never touched.
//!
//! - [`model`]: the files as written by hand.
//! - [`Content`]: the files turned into records, with every name resolved and checked.
//! - [`install`]: writing them into a database.
//! - [`help`]: the HELP topics, added to the game's text when it loads.

#![forbid(unsafe_code)]

pub mod help;
pub mod install;
pub mod model;

use silvermere_types::enums::{Ability, ArmourType, Magery, WeaponClass};
use silvermere_types::fork::{PetKind, fork_ability};
use silvermere_types::ids::{ClassNo, TextBlockNo};
use silvermere_types::records::{AbilitySlot, Class, TextBlock};
use silvermere_types::sizes::NRECABILS;

pub use install::{InstallError, InstallReport, install};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

/// The levels a title list covers, one line each (the stock lists run to about 110).
pub const TITLE_LEVELS: u16 = 110;

/// What is wrong with the content files.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    /// A file is not valid TOML of the expected shape.
    #[error("{file}: {source}")]
    Toml {
        /// The file.
        file: String,
        /// The parse error.
        source: toml::de::Error,
    },
    /// A value doesn't make sense.
    #[error("{file}: {what}: {problem}")]
    Value {
        /// The file.
        file: String,
        /// The record or field.
        what: String,
        /// What is wrong.
        problem: String,
    },
}

fn bad(file: &str, what: impl Into<String>, problem: impl Into<String>) -> ContentError {
    ContentError::Value {
        file: file.to_string(),
        what: what.into(),
        problem: problem.into(),
    }
}

/// A change to a class already in the world, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassEdit {
    /// The class's name as the world has it.
    pub name: String,
    /// The abilities to add or set.
    pub abilities: Vec<AbilitySlot>,
}

/// The content, every name resolved and every value checked.
#[derive(Debug, Clone, Default)]
pub struct Content {
    /// Classes the content adds.
    pub classes: Vec<Class>,
    /// Title blocks of those classes.
    pub titles: Vec<TextBlock>,
    /// Changes to classes already in the world.
    pub class_edits: Vec<ClassEdit>,
    /// The HELP file as written.
    pub help: model::HelpFile,
    /// Kinds of pet.
    pub pet_kinds: Vec<PetKind>,
}

impl Content {
    /// The content built into this binary.
    pub fn embedded() -> Result<Content, ContentError> {
        Content::from_files(embedded::FILES)
    }

    /// The content of the named files (`classes.toml`, `help.toml`); others are refused.
    pub fn from_files(files: &[(&str, &str)]) -> Result<Content, ContentError> {
        let mut content = Content::default();
        for &(name, text) in files {
            match name {
                "classes.toml" => {
                    let f: model::ClassesFile = parse(name, text)?;
                    content.add_classes(name, &f)?;
                }
                "help.toml" => content.help = parse(name, text)?,
                "pets.toml" => {
                    let f: model::PetsFile = parse(name, text)?;
                    content.add_pets(name, &f)?;
                }
                other => return Err(bad(other, "file", "not a known content file")),
            }
        }
        content.check_help("help.toml")?;
        Ok(content)
    }

    fn add_classes(&mut self, file: &str, f: &model::ClassesFile) -> Result<(), ContentError> {
        for c in &f.class {
            if self.classes.iter().any(|k| k.number.0 == c.number) {
                return Err(bad(file, &c.name, format!("class {} twice", c.number)));
            }
            let (class, titles) = build_class(file, c)?;
            self.classes.push(class);
            self.titles.extend(titles);
        }
        for ch in &f.class_change {
            let abilities = ch
                .add_abilities
                .iter()
                .map(|a| Ok((ability(file, &ch.name, &a.name)?, a.value)))
                .collect::<Result<Vec<_>, ContentError>>()?;
            self.class_edits.push(ClassEdit {
                name: ch.name.clone(),
                abilities,
            });
        }
        Ok(())
    }

    fn add_pets(&mut self, file: &str, f: &model::PetsFile) -> Result<(), ContentError> {
        for p in &f.pet_kind {
            let what = p.name.as_str();
            if p.number == 0 || self.pet_kinds.iter().any(|k| k.number == p.number) {
                return Err(bad(
                    file,
                    what,
                    "a pet kind's number is above 0 and used once",
                ));
            }
            if p.energy_per_blow == 0 {
                return Err(bad(file, what, "energy_per_blow is above 0"));
            }
            let (lo, hi) = p
                .damage
                .split_once('-')
                .and_then(|(a, b)| {
                    Some((a.trim().parse::<i16>().ok()?, b.trim().parse::<i16>().ok()?))
                })
                .filter(|&(lo, hi)| lo >= 0 && hi >= lo)
                .ok_or_else(|| bad(file, what, format!("damage `{}` is not like 2-8", p.damage)))?;
            if p.name.trim().is_empty()
                || p.hit_verb.trim().is_empty()
                || p.miss_verb.trim().is_empty()
            {
                return Err(bad(file, what, "a pet kind needs a name and both verbs"));
            }
            self.pet_kinds.push(PetKind {
                number: p.number,
                name: p.name.clone(),
                hit_verb: p.hit_verb.clone(),
                miss_verb: p.miss_verb.clone(),
                energy_per_blow: p.energy_per_blow,
                min_base: lo,
                min_per_level: p.per_level[0],
                max_base: hi,
                max_per_level: p.per_level[1],
                accuracy: p.accuracy,
                poison: p.poison,
            });
        }
        Ok(())
    }

    fn check_help(&self, file: &str) -> Result<(), ContentError> {
        for t in &self.help.class_topic {
            if !self.classes.iter().any(|c| c.name == t.class) {
                return Err(bad(
                    file,
                    &t.class,
                    "class_topic names no class the content adds",
                ));
            }
        }
        for t in &self.help.topic {
            if t.words.trim().is_empty() {
                return Err(bad(file, "topic", "empty words"));
            }
        }
        // the topics must also make valid help texts
        help::topics(self).map_err(|e| bad(file, "help", e))?;
        Ok(())
    }
}

fn parse<T: serde::de::DeserializeOwned>(file: &str, text: &str) -> Result<T, ContentError> {
    toml::from_str(text).map_err(|source| ContentError::Toml {
        file: file.to_string(),
        source,
    })
}

/// An ability by its content name: a fork ability, or a stock one by its variant name.
fn ability(file: &str, what: &str, name: &str) -> Result<Ability, ContentError> {
    if let Some(a) = fork_ability(name) {
        return Ok(a);
    }
    Ability::ALL
        .iter()
        .copied()
        .find(|a| format!("{a:?}").eq_ignore_ascii_case(name))
        .filter(|&a| a != Ability::None)
        .ok_or_else(|| bad(file, what, format!("unknown ability `{name}`")))
}

fn by_name<T: Copy + std::fmt::Debug>(all: &[T], name: &str) -> Option<T> {
    all.iter()
        .copied()
        .find(|v| format!("{v:?}").eq_ignore_ascii_case(name))
}

fn build_class(
    file: &str,
    c: &model::NewClass,
) -> Result<(Class, Option<TextBlock>), ContentError> {
    let what = c.name.as_str();
    if c.name.is_empty() || c.name.len() > 29 {
        return Err(bad(file, what, "a class name is 1-29 characters"));
    }
    if c.number == 0 {
        return Err(bad(file, what, "class number 0 is not a class"));
    }
    let (lo, hi) =
        c.hp.split_once('-')
            .and_then(|(a, b)| Some((a.trim().parse::<i16>().ok()?, b.trim().parse::<i16>().ok()?)))
            .filter(|&(lo, hi)| lo >= 1 && hi >= lo)
            .ok_or_else(|| bad(file, what, format!("hp `{}` is not a range like 5-9", c.hp)))?;
    if !(1..=1000).contains(&c.exp) {
        return Err(bad(file, what, "exp is a percentage, 100 for a Warrior's"));
    }
    if !(1..=5).contains(&c.fighting) {
        return Err(bad(file, what, "fighting is 1-5"));
    }
    let (magery, magelvl) = match c.magic.split_once('-') {
        Some((m, l)) => (
            by_name(Magery::ALL, m.trim()),
            l.trim().parse::<i16>().ok().filter(|l| (1..=3).contains(l)),
        ),
        None if c.magic.eq_ignore_ascii_case("none") => (Some(Magery::None), Some(0)),
        None => (None, None),
    };
    let (Some(magery), Some(magelvl)) = (magery, magelvl) else {
        return Err(bad(
            file,
            what,
            format!("magic `{}` is not like Mage-1", c.magic),
        ));
    };
    let weapon = by_name(WeaponClass::ALL, &c.weapons)
        .ok_or_else(|| bad(file, what, format!("unknown weapon class `{}`", c.weapons)))?;
    let armour = by_name(ArmourType::ALL, &c.armour)
        .ok_or_else(|| bad(file, what, format!("unknown armour type `{}`", c.armour)))?;
    if c.abilities.len() > NRECABILS {
        return Err(bad(file, what, format!("at most {NRECABILS} abilities")));
    }
    let mut abilities = [(Ability::None, 0i16); NRECABILS];
    for (slot, a) in abilities.iter_mut().zip(&c.abilities) {
        *slot = (ability(file, what, &a.name)?, a.value);
    }
    let titles = c
        .titles
        .as_ref()
        .map(|t| build_titles(file, what, t))
        .transpose()?;
    let class = Class {
        number: ClassNo(c.number),
        name: c.name.clone(),
        basehp: lo,
        hplevel: hi - lo,
        exp: c.exp - 100,
        attmult: 0,
        maxenergy: 0,
        restored: 0,
        abilities,
        magery,
        magelvl,
        weapon,
        armour,
        combat: c.fighting + 2,
        titles: titles.as_ref().map_or(TextBlockNo(0), |t| t.number),
    };
    Ok((class, titles))
}

fn build_titles(file: &str, what: &str, t: &model::Titles) -> Result<TextBlock, ContentError> {
    if t.block <= 0 {
        return Err(bad(file, what, "a titles block number is above 0"));
    }
    let Some(first) = t.ranks.first() else {
        return Err(bad(file, what, "titles with no ranks"));
    };
    if first.from != 1 {
        return Err(bad(file, what, "the first rank starts at level 1"));
    }
    if t.ranks.windows(2).any(|w| w[1].from <= w[0].from) {
        return Err(bad(file, what, "ranks go up in level"));
    }
    if t.ranks
        .iter()
        .any(|r| r.title.trim().is_empty() || r.title.contains('\n'))
    {
        return Err(bad(file, what, "a rank needs a one-line title"));
    }
    let mut lines = Vec::with_capacity(usize::from(TITLE_LEVELS));
    for level in 1..=TITLE_LEVELS {
        let rank = t
            .ranks
            .iter()
            .rev()
            .find(|r| r.from <= level)
            .expect("rank 1 exists");
        lines.push(rank.title.trim().to_string());
    }
    let mut text = lines.join("\n");
    text.push('\n');
    if text.len() >= 2000 {
        return Err(bad(
            file,
            what,
            "the titles don't fit one text block (2000 bytes)",
        ));
    }
    Ok(TextBlock {
        part: 0,
        number: TextBlockNo(t.block),
        link: TextBlockNo(0),
        text,
    })
}
