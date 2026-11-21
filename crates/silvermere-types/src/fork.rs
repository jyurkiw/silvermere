//! The fork's own codes: abilities that stock MajorMUD 1.11p doesn't have.
//!
//! They are `Ability::Unknown` codes in a range far above stock's last ability (187), so
//! the stock enum and its committed facts stay as they are, and an ability upstream adds
//! later can't collide with one of these. Content (`content/`) names them by
//! [`FORK_ABILITIES`]; the engine matches on the constants.

use serde::{Deserialize, Serialize};

use crate::enums::Ability;

/// Spellstrike: a single-target attack spell cast with a weapon wielded takes the place of
/// the round's first blow. The value is the failure discount, in percent of a blow's cost.
pub const SPELLSTRIKE: Ability = Ability::Unknown(1001);
/// The War Mage's extra discount on a successful spellstrike, in percent. Hidden.
pub const SPELLSTRIKE_SUCCESS_OFF: Ability = Ability::Unknown(1002);
/// The War Mage's extra discount on a failed spellstrike, in percent. Hidden.
pub const SPELLSTRIKE_FAIL_OFF: Ability = Ability::Unknown(1003);
/// The class may not use items worn off-hand (shields).
pub const NO_OFF_HAND: Ability = Ability::Unknown(1004);
/// On a character: the kind of the companion it has bonded (design/warden.md); 0 or
/// absent for none. Set by content scripts (`addability`).
pub const PET_KIND: Ability = Ability::Unknown(1005);
/// On a class: from this level the class may wear chainmail.
pub const CHAINMAIL_AT_LEVEL: Ability = Ability::Unknown(1006);
/// On a class: from this level the class may wear scalemail.
pub const SCALEMAIL_AT_LEVEL: Ability = Ability::Unknown(1007);
/// On an item: a ranged weapon (a sling, a bow). For now only a marker for who may use it.
pub const RANGED: Ability = Ability::Unknown(1008);
/// On a class: may use any item marked [`RANGED`], whatever its weapon type.
pub const RANGED_WEAPONS: Ability = Ability::Unknown(1009);

/// The fork's abilities by the name content uses, and whether players are shown them.
pub const FORK_ABILITIES: &[(&str, Ability, bool)] = &[
    ("Spellstrike", SPELLSTRIKE, false),
    ("SpellstrikeSuccessOff", SPELLSTRIKE_SUCCESS_OFF, true),
    ("SpellstrikeFailOff", SPELLSTRIKE_FAIL_OFF, true),
    ("NoOffHand", NO_OFF_HAND, false),
    ("PetKind", PET_KIND, true),
    ("ChainmailAtLevel", CHAINMAIL_AT_LEVEL, false),
    ("ScalemailAtLevel", SCALEMAIL_AT_LEVEL, false),
    ("Ranged", RANGED, false),
    ("RangedWeapons", RANGED_WEAPONS, false),
];

/// A fork ability by its content name.
pub fn fork_ability(name: &str) -> Option<Ability> {
    FORK_ABILITIES
        .iter()
        .find(|(n, _, _)| n.eq_ignore_ascii_case(name))
        .map(|&(_, a, _)| a)
}

/// The content name of a fork ability, if `ability` is one.
pub fn fork_ability_name(ability: Ability) -> Option<&'static str> {
    FORK_ABILITIES
        .iter()
        .find(|&&(_, a, _)| a == ability)
        .map(|&(n, _, _)| n)
}

/// Whether `ability` is one of the fork's hidden abilities, never shown to players.
pub fn is_hidden_ability(ability: Ability) -> bool {
    FORK_ABILITIES
        .iter()
        .any(|&(_, a, hidden)| hidden && a == ability)
}

/// A kind of pet (design/warden.md): a hunting hound, a hawkling, ... Content
/// (`content/pets.toml`) defines them; a character's [`PET_KIND`] names one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetKind {
    /// The kind's number, the value of a character's [`PET_KIND`].
    pub number: u16,
    /// What it's called before it has a name: "hunting hound".
    pub name: String,
    /// The verb of a hit: "bites" ("Fang bites the rat for 9 damage!").
    pub hit_verb: String,
    /// The verb of a miss: "snaps at" ("Fang snaps at the rat!").
    pub miss_verb: String,
    /// The energy one blow costs; a round gives 1000 (1000: a blow a round, 333: three,
    /// 2000: one every other round).
    pub energy_per_blow: u16,
    /// Least damage at level 0, and what each level adds, in hundredths.
    pub min_base: i16,
    pub min_per_level: i16,
    /// Most damage at level 0, and what each level adds, in hundredths.
    pub max_base: i16,
    pub max_per_level: i16,
    /// Added to the owner's attack value for the pet's blows.
    pub accuracy: i16,
    /// Poison a hit leaves on the target (0 none).
    pub poison: i16,
}

impl PetKind {
    /// The damage range of a blow for an owner of `level`.
    pub fn damage(&self, level: u16) -> (i32, i32) {
        let at = |base: i16, per: i16| i32::from(base) + i32::from(level) * i32::from(per) / 100;
        let max = at(self.max_base, self.max_per_level).max(0);
        (at(self.min_base, self.min_per_level).clamp(0, max), max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fork_codes_are_not_stock_codes() {
        for &(name, a, _) in FORK_ABILITIES {
            assert!(
                matches!(a, Ability::Unknown(_)),
                "{name} must not be a stock ability"
            );
            assert!(!Ability::ALL.iter().any(|s| s.code() == a.code()));
        }
    }

    #[test]
    fn names_round_trip() {
        for &(name, a, _) in FORK_ABILITIES {
            assert_eq!(fork_ability(name), Some(a));
            assert_eq!(fork_ability(&name.to_lowercase()), Some(a));
            assert_eq!(fork_ability_name(a), Some(name));
        }
        assert!(is_hidden_ability(SPELLSTRIKE_FAIL_OFF));
        assert!(!is_hidden_ability(SPELLSTRIKE));
    }

    // design/warden.md: the hunting hound, 2-10 at level 1 to 17-68 at level 30
    #[test]
    fn pet_damage_grows_with_level() {
        let hound = PetKind {
            number: 1,
            name: "hunting hound".into(),
            hit_verb: "bites".into(),
            miss_verb: "snaps at".into(),
            energy_per_blow: 1000,
            min_base: 2,
            min_per_level: 50,
            max_base: 8,
            max_per_level: 200,
            accuracy: 0,
            poison: 0,
        };
        assert_eq!(hound.damage(1), (2, 10));
        assert_eq!(hound.damage(10), (7, 28));
        assert_eq!(hound.damage(20), (12, 48));
        assert_eq!(hound.damage(30), (17, 68));
    }
}
