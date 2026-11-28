//! Fork: the Dervish's two hands (design/dervish.md).
//!
//! A class with `DualWield` may hold a second one-handed weapon in a worn slot (OFFHAND).
//! In an auto-combat round of plain attacks on a monster, the two hands take turns: each
//! has its own energy pool (the main hand the character's own, the off-hand a pool kept
//! here, a round's energy added each round), and each blow costs that hand's weapon's
//! energy. A hand that can't pay drops out and the other goes on. Each hand strikes at
//! least once a round, whatever its weapon costs; the two make at most
//! [`DUAL_MAX_BLOWS`] together, and neither more than the stock [`MAX_BLOWS`]. The off-hand
//! makes at most [`offhand_cap`] blows: 1 below level 12, 2 below level 20, and from level 20
//! the blows the main hand's energy leaves short of the 8. The off-hand's blow is a stock blow with the off-hand weapon, its attack value less the
//! ability's value.

use silvermere_rules::combat::{AlignGroup, Fighter, MAX_BLOWS, Move, TICK_ENERGY};
use silvermere_text::Msg;
use silvermere_types::enums::ItemType;
use silvermere_types::fork;
use silvermere_types::ids::{ItemNo, SessionId};
use silvermere_types::records::Character;
use silvermere_types::sizes::NWORN;

use super::{Verbs, player, player_mut, player_side, room_of, user_ability, user_has};
use crate::tell::{Noise, TellBlind, tell_room, tell_user};
use crate::world::World;

/// The most blows a round makes with both hands.
pub const DUAL_MAX_BLOWS: u32 = 8;

/// The level from which the off-hand makes 2 blows a round rather than 1.
pub const OFFHAND_TWO_BLOWS_LEVEL: u16 = 12;
/// The level from which the off-hand fills in the blows the main hand doesn't make.
pub const OFFHAND_FILL_LEVEL: u16 = 20;

/// The most blows the off-hand makes in a round for a Dervish of `level` whose main hand
/// makes `main_blows`: 1 below [`OFFHAND_TWO_BLOWS_LEVEL`], 2 below [`OFFHAND_FILL_LEVEL`],
/// and from then on what the main hand leaves of [`DUAL_MAX_BLOWS`]. Never more than the
/// stock [`MAX_BLOWS`], and never less than the one blow each hand always makes.
pub fn offhand_cap(level: u16, main_blows: u32) -> u32 {
    let by_level = if level < OFFHAND_TWO_BLOWS_LEVEL {
        1
    } else if level < OFFHAND_FILL_LEVEL {
        2
    } else {
        MAX_BLOWS
    };
    by_level
        .min(DUAL_MAX_BLOWS.saturating_sub(main_blows))
        .clamp(1, MAX_BLOWS)
}

/// The blows the main hand's `energy` pays for at `cost` a blow: at least 1, at most the
/// stock [`MAX_BLOWS`].
fn main_blows(energy: i32, cost: i32) -> u32 {
    (energy.max(0) / cost.max(1)).clamp(1, MAX_BLOWS as i32) as u32
}

/// The off-hand weapon a character holds: the worn slot and the item, the first weapon
/// among its worn items.
pub fn offhand_weapon(world: &World, plr: &Character) -> Option<(usize, ItemNo)> {
    plr.worn
        .iter()
        .take(NWORN)
        .enumerate()
        .find(|&(_, w)| {
            w.0 != 0
                && world
                    .items
                    .get(w)
                    .is_some_and(|i| i.item_type == ItemType::Weapon)
        })
        .map(|(i, &w)| (i, w))
}

/// Where a worn item is shown worn: its location's name, and for a weapon (which can only be
/// worn as an off-hand weapon, its own place being "Nowhere") the shield's place, "Off-Hand".
pub fn worn_place(itm: &silvermere_types::records::Item) -> &'static str {
    if itm.item_type == ItemType::Weapon {
        silvermere_types::enums::WornLocation::OffHand
            .name()
            .unwrap_or("Off-Hand")
    } else {
        itm.worn.name().unwrap_or_default()
    }
}

/// Which hand strikes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

/// The off-hand's side of a round of blows, and the count of both hands' blows.
pub struct Hands {
    /// The off-hand weapon.
    pub item: ItemNo,
    /// The off-hand's fighter for its next blow, and its verbs.
    pub atk: Fighter,
    pub verbs: Verbs,
    /// The off-hand's energy left this round.
    pub energy: i32,
    penalty: i32,
    /// The character's level, for [`offhand_cap`].
    level: u16,
    /// The off-hand's most blows this round, set at the round's first blow.
    off_cap: Option<u32>,
    blows: [u32; 2],
    last: Option<Hand>,
}

impl Hands {
    /// The off-hand's part in this round of `mv` by `plr`, when it has one: a class with
    /// `DualWield`, a weapon in each hand, and a plain attack.
    pub fn start(world: &mut World, id: SessionId, plr: &Character, mv: Move) -> Option<Self> {
        if mv != Move::Attack || plr.weapon.0 == 0 || !user_has(world, plr, fork::DUAL_WIELD) {
            return None;
        }
        let (_, item) = offhand_weapon(world, plr)?;
        let penalty = user_ability(world, plr, fork::DUAL_WIELD);
        let carry = world.offhand_energy.get(&id).copied().unwrap_or(0);
        let mut hands = Self {
            item,
            atk: Fighter::default(),
            verbs: Verbs::default(),
            energy: carry + TICK_ENERGY,
            penalty,
            level: plr.level,
            off_cap: None,
            blows: [0, 0],
            last: None,
        };
        hands.refresh(world, id)?;
        Some(hands)
    }

    /// The off-hand's fighter again, for its next blow: the character's side with the
    /// off-hand weapon in hand, less the off-hand's penalty.
    pub fn refresh(&mut self, world: &mut World, id: SessionId) -> Option<()> {
        let mut plr = player(world, id)?.clone();
        plr.weapon = self.item;
        let (mut atk, verbs) = player_side(
            world,
            id,
            &plr,
            Move::Attack,
            Move::Attack,
            AlignGroup::Neutral,
        )?;
        atk.av -= self.penalty;
        self.atk = atk;
        self.verbs = verbs;
        Some(())
    }

    fn can(&self, hand: Hand, energy: i32, cost: i32) -> bool {
        let n = self.blows[hand as usize];
        let cap = match hand {
            Hand::Main => MAX_BLOWS,
            Hand::Off => self.off_cap.unwrap_or(1),
        };
        n < cap && (n == 0 || energy >= cost)
    }

    /// The hand to strike next, the main hand's `main_energy` and `main_cost` given: the hands
    /// take turns, a hand that can't pay is passed over, and `None` ends the round.
    pub fn next(&mut self, main_energy: i32, main_cost: i32) -> Option<Hand> {
        if self.blows[0] + self.blows[1] >= DUAL_MAX_BLOWS {
            return None;
        }
        // the off-hand's share, from the main hand's energy at the round's start
        if self.off_cap.is_none() {
            self.off_cap = Some(offhand_cap(self.level, main_blows(main_energy, main_cost)));
        }
        let main = self.can(Hand::Main, main_energy, main_cost);
        let off = self.can(Hand::Off, self.energy, self.atk.energy as i32);
        let hand = match (self.last, main, off) {
            (Some(Hand::Main), _, true) | (_, false, true) => Hand::Off,
            (_, true, _) => Hand::Main,
            _ => return None,
        };
        self.blows[hand as usize] += 1;
        self.last = Some(hand);
        Some(hand)
    }

    /// The off-hand pays for its blow; a first blow it couldn't afford leaves it nothing.
    pub fn pay(&mut self, cost: i32) {
        self.energy = (self.energy - cost).max(0);
    }

    /// The round is over: the off-hand's energy carries over to the next.
    pub fn finish(self, world: &mut World, id: SessionId) {
        world
            .offhand_energy
            .insert(id, self.energy.min(i32::from(u16::MAX)));
    }
}

/// When the main hand is empty and the off-hand holds a weapon, the weapon moves to the
/// main hand (design/dervish.md): REMOVE, a drop, a theft. Told to the character.
pub fn settle_hands(world: &mut World, id: SessionId) {
    let Some(plr) = player(world, id) else {
        return;
    };
    if plr.weapon.0 != 0 {
        return;
    }
    let Some((slot, item)) = offhand_weapon(world, plr) else {
        return;
    };
    // an off-hand-only weapon stays where it is (design/class-trainers.md)
    if world
        .items
        .get(&item)
        .is_some_and(|i| silvermere_rules::abilities::item_has_ability(i, fork::OFFHAND_ONLY))
    {
        return;
    }
    let name = world
        .items
        .get(&item)
        .map(|i| i.name.clone())
        .unwrap_or_default();
    let Some(p) = player_mut(world, id) else {
        return;
    };
    p.worn[slot] = ItemNo(0);
    p.wornuses[slot] = 0;
    p.weapon = item;
    world.offhand_energy.remove(&id);
    tell_user(world, id, Msg::OffhandShifted { item: name });
}

/// OFFHAND's room line, as a wield's.
pub fn tell_room_wields(world: &mut World, id: SessionId, item: &str, his: &str) {
    let Some(p) = player(world, id) else {
        return;
    };
    let who = p.firstname.clone();
    let room = room_of(p);
    tell_room(
        world,
        room,
        [Some(id), None],
        Noise::Always,
        TellBlind::All,
        false,
        None,
        &Msg::OffhandOtherWields {
            who,
            item: item.to_owned(),
            his: his.to_owned(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_off_hand_cap_grows_with_level() {
        assert_eq!(offhand_cap(1, 1), 1);
        assert_eq!(offhand_cap(11, 3), 1);
        assert_eq!(offhand_cap(12, 3), 2);
        assert_eq!(offhand_cap(19, 1), 2);
        // from 20, the blows the main hand leaves of the 8, at most 6
        assert_eq!(offhand_cap(20, 6), 2);
        assert_eq!(offhand_cap(20, 5), 3);
        assert_eq!(offhand_cap(20, 1), 6);
        assert_eq!(offhand_cap(60, 8), 1);
    }

    #[test]
    fn the_main_hand_blows_come_from_its_energy() {
        assert_eq!(main_blows(1000, 200), 5);
        assert_eq!(main_blows(1200, 200), 6);
        assert_eq!(main_blows(2000, 200), 6);
        assert_eq!(main_blows(1000, 1500), 1);
        assert_eq!(main_blows(0, 0), 1);
    }
}
