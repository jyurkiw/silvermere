//! Spellstrike's arithmetic (fork, design/war-mage.md): the cast's energy and the speed
//! bonus's split between the cast and the blows. Pure.
//!
//! In a spellstrike round the cast takes the place of the first blow and costs that
//! blow's energy, less a discount: the class's success or failure discount, in percent.
//! A fast blow's critical bonus goes to the cast chance first, up to the cast cap; only
//! what would pass the cap stays with the blows as critical chance.

use crate::spells::MAXCHANCE;

/// A class's Spellstrike discounts, in percent of a blow's energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Discounts {
    /// Off a successful cast.
    pub success_off: i32,
    /// Off a failed (or resisted) cast.
    pub fail_off: i32,
}

impl Discounts {
    /// From the ability values: `Spellstrike` (the base failure discount) plus the
    /// class's extra success and failure discounts.
    pub fn from_abilities(spellstrike: i32, success_off: i32, fail_off: i32) -> Self {
        Self {
            success_off: success_off.clamp(0, 100),
            fail_off: (spellstrike + fail_off).clamp(0, 100),
        }
    }

    /// The cast's energy for a blow of `blow` energy, rounded down.
    pub fn cast_energy(self, blow: u32, success: bool) -> u32 {
        let off = if success {
            self.success_off
        } else {
            self.fail_off
        };
        blow * (100 - off.clamp(0, 100)) as u32 / 100
    }
}

/// A fast blow's critical bonus, split between the cast and the blows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeedSplit {
    /// Added to the cast chance.
    pub to_cast: i32,
    /// Left with the blows as critical chance.
    pub to_crit: i32,
}

/// Splits `bonus` for a cast whose chance before it is `chance` (Spellcasting plus the
/// spell's difficulty): the cast takes what fits below [`MAXCHANCE`], the blows the rest.
pub fn split_speed_bonus(chance: i32, bonus: i32) -> SpeedSplit {
    let bonus = bonus.max(0);
    let room = (MAXCHANCE - chance.min(MAXCHANCE)).max(0);
    let to_cast = bonus.min(room);
    SpeedSplit {
        to_cast,
        to_crit: bonus - to_cast,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const WAR_MAGE: Discounts = Discounts {
        success_off: 10,
        fail_off: 70,
    };

    // design/war-mage.md, Worked example: blows of 256, 342 and 200 energy
    #[test]
    fn the_worked_example() {
        assert_eq!(Discounts::from_abilities(30, 10, 40), WAR_MAGE);
        assert_eq!(WAR_MAGE.cast_energy(256, true), 230);
        assert_eq!(WAR_MAGE.cast_energy(256, false), 76);
        assert_eq!(WAR_MAGE.cast_energy(342, true), 307);
        assert_eq!(WAR_MAGE.cast_energy(342, false), 102);
        assert_eq!(WAR_MAGE.cast_energy(200, true), 180);
        assert_eq!(WAR_MAGE.cast_energy(200, false), 60);
        // a warlock with Spellstrike alone: full price on success, 30% off a failure
        let warlock = Discounts::from_abilities(30, 0, 0);
        assert_eq!(warlock.cast_energy(200, true), 200);
        assert_eq!(warlock.cast_energy(200, false), 140);
    }

    // design/war-mage.md, Speed bonus to the cast: 90% with +20 casts at 98% and keeps +12
    #[test]
    fn the_speed_bonus_goes_to_the_cast_first() {
        assert_eq!(
            split_speed_bonus(90, 20),
            SpeedSplit {
                to_cast: 8,
                to_crit: 12
            }
        );
        assert_eq!(
            split_speed_bonus(60, 20),
            SpeedSplit {
                to_cast: 20,
                to_crit: 0
            }
        );
        assert_eq!(
            split_speed_bonus(120, 7),
            SpeedSplit {
                to_cast: 0,
                to_crit: 7
            }
        );
        assert_eq!(
            split_speed_bonus(40, 0),
            SpeedSplit {
                to_cast: 0,
                to_crit: 0
            }
        );
    }

    proptest! {
        #[test]
        fn a_cast_never_costs_more_than_its_blow(
            blow in 0u32..=1000,
            s in -50i32..150,
            f in -50i32..150,
            b in -50i32..150,
            success in any::<bool>(),
        ) {
            let d = Discounts::from_abilities(b, s, f);
            prop_assert!(d.cast_energy(blow, success) <= blow);
        }

        #[test]
        fn the_split_adds_up_and_respects_the_cap(chance in -100i32..200, bonus in -10i32..40) {
            let s = split_speed_bonus(chance, bonus);
            prop_assert_eq!(s.to_cast + s.to_crit, bonus.max(0));
            prop_assert!(s.to_cast >= 0 && s.to_crit >= 0);
            prop_assert!(chance.min(MAXCHANCE) + s.to_cast <= MAXCHANCE.max(chance.min(MAXCHANCE)));
        }
    }
}
