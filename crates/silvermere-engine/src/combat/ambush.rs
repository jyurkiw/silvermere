//! Fork: spell backstab (design/stealth-casters.md).
//!
//! A class with `SpellBackstab` that casts a single-target attack spell from hiding or
//! sneaking makes a stealth roll as the spell is begun: the stock hide/sneak chance less
//! the class's `SpellBackstabStealth` penalty. The roll is kept until the cast itself (the
//! first auto-combat round). If the caster stayed unseen and the spell lands, its amount is
//! built as a melee backstab's damage is ([`backstab_amount`]), so a spell backstab keeps
//! pace with a weapon's as the levels go up. One attempt: whatever happens, the caster is
//! revealed.

use silvermere_rules::Rng;
use silvermere_text::Msg;
use silvermere_types::enums::Ability;
use silvermere_types::fork;
use silvermere_types::ids::SessionId;
use silvermere_types::records::Spell;

use super::spellstrike::is_spellstrike_spell;
use super::{player, room_of, user_ability, user_has};
use crate::movement::move_user::calc_hide_sneak_chance;
use crate::tell::{Noise, TellBlind, tell_room, tell_user};
use crate::world::World;

/// The stealth roll, as a CAST of `spl` at a player or monster is begun by a caster who
/// was hidden or sneaking (`from_stealth`). Does nothing unless this is a spell backstab.
pub fn prepare(world: &mut World, id: SessionId, spl: &Spell, from_stealth: bool) {
    world.ambush.remove(&id);
    let Some(plr) = player(world, id) else {
        return;
    };
    // only a spell the CAST command engages with, so that its cast comes in the next round
    if !from_stealth
        || !is_spellstrike_spell(spl)
        || spl.duration != 0
        || !user_has(world, plr, fork::SPELL_BACKSTAB)
    {
        return;
    }
    let penalty = user_ability(world, plr, fork::SPELL_BACKSTAB_STEALTH);
    let chance = (calc_hide_sneak_chance(world, id, 95, Some(id)) - penalty).max(0);
    let kept = world.rng.range(0, 100) < chance;
    if !kept {
        tell_user(world, id, Msg::SpellBackstabSpotted);
    }
    world.ambush.insert(id, kept);
}

/// Takes the waiting stealth roll at the cast: `true` when it is a spell backstab.
pub fn take(world: &mut World, id: SessionId) -> bool {
    world.ambush.remove(&id) == Some(true)
}

/// A spell backstab's lead-in lines, and `rolled` (the spell's rolled amount) raised.
pub fn strike(world: &mut World, id: SessionId, spl: &Spell, rolled: i32) -> i32 {
    let Some(p) = player(world, id) else {
        return rolled;
    };
    let pct = user_ability(world, p, fork::SPELL_BACKSTAB);
    let min = user_ability(world, p, Ability::BackstabMin);
    let max = user_ability(world, p, Ability::BackstabMax).max(min);
    let (level, stealth) = (i32::from(p.level), i32::from(p.stealth));
    let who = p.firstname.clone();
    let room = room_of(p);
    tell_user(
        world,
        id,
        Msg::SpellBackstabLeadIn {
            spell: spl.name.clone(),
        },
    );
    tell_room(
        world,
        room,
        [Some(id), None],
        Noise::Medium,
        TellBlind::All,
        false,
        None,
        &Msg::SpellBackstabLeadInRoom {
            who,
            spell: spl.name.clone(),
        },
    );
    let bonus = world.rng.range(min, max + 1);
    backstab_amount(rolled, pct, level, stealth, bonus)
}

/// A spell backstab's amount, built as the stock melee backstab builds its damage with the
/// spell's roll in the weapon's place: the roll raised by `pct` (100 doubles it), plus twice
/// the level, a tenth of the stealth and the Backstab Min./Max. roll, then all of it raised
/// by the level in percent.
pub fn backstab_amount(rolled: i32, pct: i32, level: i32, stealth: i32, bonus: i32) -> i32 {
    let base = i64::from(rolled) * i64::from(100 + pct) / 100
        + i64::from(level) * 2
        + i64::from(stealth) / 10
        + i64::from(bonus);
    (base * (i64::from(level) + 100) / 100).clamp(0, i64::from(i32::MAX)) as i32
}
