//! Fork: the Spellstrike round (design/war-mage.md).
//!
//! A character whose class has `Spellstrike`, fighting a monster in auto-combat with an
//! attack spell, with a weapon wielded and a single-target attack spell, makes a
//! spellstrike round instead of a casting round:
//!
//! 1. the cast goes first, in place of the first blow, with the fast-blow critical bonus
//!    added to its chance up to the cast cap;
//! 2. it costs one blow's energy less the class's success or failure discount (a resisted
//!    cast counts as a failure), and mana as stock charges it;
//! 3. the rest of the round's energy buys blows as a normal round would, without the
//!    after-cast extra, and with only the bonus left over from the cast as critical chance;
//! 4. out of mana, the fight carries on as plain melee, and stays so until the player
//!    casts again.
//!
//! Anything else (no weapon, an area spell, a player target) falls through to the stock
//! casting round. Classes without the ability never get here.

use silvermere_rules::combat::spellstrike::{Discounts, split_speed_bonus};
use silvermere_rules::combat::{AlignGroup, Move};
use silvermere_text::Msg;
use silvermere_types::enums::{Ability, SpellTarget, SpellType};
use silvermere_types::flags::PlayerSettings;
use silvermere_types::fork;
use silvermere_types::ids::{ActiveMonsterId, SessionId, SpellNo};
use silvermere_types::records::Spell;

use super::autocombat::{keep_one_queued, resume_casting};
use super::{
    AutoTarget, Mode, attack_user_monster, monster, player, player_mut, player_side,
    possessive_pronoun, room_of, user_ability, user_has,
};
use crate::tell::{Noise, TellBlind, tell_room, tell_user};
use crate::world::World;

/// Whether `spl` can be cast as a spellstrike: a single-target attack spell that does
/// damage and costs energy.
pub fn is_spellstrike_spell(spl: &Spell) -> bool {
    spl.spell_type == SpellType::Autocombat
        && matches!(
            spl.target,
            SpellTarget::Single | SpellTarget::Monster | SpellTarget::MonsterOrSingle
        )
        && spl.energy > 0
        && spl.abilities.iter().any(|&(a, _)| {
            matches!(
                a,
                Ability::DamageNoArmour | Ability::DamageArmour | Ability::StealHp
            )
        })
}

/// One auto-combat round of `spell` at monster `mon` as a spellstrike, when it is one.
/// Returns `false`, having done nothing, when the stock casting round should run instead.
pub fn try_round(world: &mut World, id: SessionId, spell: SpellNo, mon: ActiveMonsterId) -> bool {
    let Some(plr) = player(world, id).cloned() else {
        return false;
    };
    let Some(spl) = world.spells.get(&spell).cloned() else {
        return false;
    };
    if plr.weapon.0 == 0 || !is_spellstrike_spell(&spl) || !user_has(world, &plr, fork::SPELLSTRIKE)
    {
        return false;
    }
    let target = AutoTarget::Monster(mon);
    // the stock checks a round makes before anything: a full pool, alive, unafraid
    if plr.energy < plr.maxenergy || plr.hp <= 0 || monster(world, mon).is_none() {
        return false;
    }
    // out of mana: plain melee from now on, until the player casts again
    if plr.mana < spl.mana {
        tell_user(
            world,
            id,
            Msg::CastShortMana {
                what: "mana to cast that spell".to_owned(),
            },
        );
        attack_user_monster(world, id, mon, Move::Attack, Move::Attack, Mode::Auto);
        return true;
    }
    let discounts = Discounts::from_abilities(
        user_ability(world, &plr, fork::SPELLSTRIKE),
        user_ability(world, &plr, fork::SPELLSTRIKE_SUCCESS_OFF),
        user_ability(world, &plr, fork::SPELLSTRIKE_FAIL_OFF),
    );
    // the blow the cast replaces, priced as a first blow of the round (no after-cast extra)
    let mut fresh = plr.clone();
    fresh.settings.insert(PlayerSettings::CANCAST);
    let Some((blow, _)) = player_side(
        world,
        id,
        &fresh,
        Move::Attack,
        Move::Attack,
        AlignGroup::Neutral,
    ) else {
        return false;
    };
    let split = split_speed_bonus(
        i32::from(plr.spellcast) + i32::from(spl.diff),
        blow.fast_bonus,
    );

    lead_in(world, id, &spl);

    // the cast: exactly one, with the speed bonus on the chance
    let (energy, mana) = (plr.energy, plr.mana);
    if let Some(p) = player_mut(world, id) {
        p.energy = spl.energy;
        p.spellcast = p.spellcast.saturating_add(split.to_cast as i16);
    }
    let r = crate::spells::autocombat_cast(world, id, spell, Some(target));
    let Some(p) = player_mut(world, id) else {
        return true;
    };
    p.spellcast = plr.spellcast;
    // a failed or resisted cast charges half the mana, a success all of it
    let charged = i32::from(mana) - i32::from(p.mana);
    let success = spl.mana <= 0 || charged >= i32::from(spl.mana);
    let refused = r.outcome == crate::spells::cast::CastOutcome::Refused;
    let cost = if refused {
        0
    } else {
        discounts.cast_energy(blow.energy, success)
    };
    p.energy = energy.saturating_sub(cost.min(u32::from(u16::MAX)) as u16);

    // the blows, with what the round has left
    if !refused && monster(world, mon).is_some() && player(world, id).is_some_and(|p| p.hp > 0) {
        let settings = player(world, id).map(|p| p.settings);
        if let Some(p) = player_mut(world, id) {
            p.settings.insert(PlayerSettings::CANCAST);
        }
        world.spellstrike_crit_cut = Some(split.to_cast);
        attack_user_monster(world, id, mon, Move::Attack, Move::Attack, Mode::Auto);
        world.spellstrike_crit_cut = None;
        if let (Some(p), Some(s)) = (player_mut(world, id), settings) {
            p.settings = s;
        }
    }
    // one place in the ring, and the fight carries on as spellstrikes
    keep_one_queued(world, id);
    resume_casting(world, id, spell);
    true
}

/// The line before the cast, to the caster and the room.
fn lead_in(world: &mut World, id: SessionId, spl: &Spell) {
    let Some(p) = player(world, id) else {
        return;
    };
    let weapon = world
        .items
        .get(&p.weapon)
        .map(|i| i.name.clone())
        .unwrap_or_default();
    let who = p.firstname.clone();
    let his = possessive_pronoun(Some(p)).to_owned();
    let room = room_of(p);
    tell_user(
        world,
        id,
        Msg::SpellstrikeLeadIn {
            spell: spl.name.clone(),
            weapon: weapon.clone(),
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
        &Msg::SpellstrikeLeadInRoom {
            who,
            spell: spl.name.clone(),
            his,
            weapon,
        },
    );
}
