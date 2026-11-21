//! The fork's own codes: abilities that stock MajorMUD 1.11p doesn't have.
//!
//! They are `Ability::Unknown` codes in a range far above stock's last ability (187), so
//! the stock enum and its committed facts stay as they are, and an ability upstream adds
//! later can't collide with one of these. Content (`content/`) names them by
//! [`FORK_ABILITIES`]; the engine matches on the constants.

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

/// The fork's abilities by the name content uses, and whether players are shown them.
pub const FORK_ABILITIES: &[(&str, Ability, bool)] = &[
    ("Spellstrike", SPELLSTRIKE, false),
    ("SpellstrikeSuccessOff", SPELLSTRIKE_SUCCESS_OFF, true),
    ("SpellstrikeFailOff", SPELLSTRIKE_FAIL_OFF, true),
    ("NoOffHand", NO_OFF_HAND, false),
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
}
