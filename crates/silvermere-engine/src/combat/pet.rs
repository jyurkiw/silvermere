//! Fork: a companion's blows (design/warden.md).
//!
//! At the start of a character's auto-combat round against a monster, before the character's
//! own blows, its companion strikes that monster on its own: blows paid for from the pet's own energy (a full round each
//! round, carried over like a fighter's), so a kind's `energy_per_blow` sets how often it
//! strikes. A blow is a fighter's blow: the owner's attack value plus the kind's accuracy,
//! the kind's damage for the owner's level, against the monster's defence, through the
//! stock blow resolution. It costs the owner nothing. A kill counts as the owner's.
//!
//! The first version's companion shares its owner's hit points, so monsters keep attacking
//! only the owner.

use silvermere_rules::combat::{AlignGroup, MAX_BLOWS, Move, Outcome, TICK_ENERGY, resolve};
use silvermere_text::Msg;
use silvermere_types::fork::{self, PetKind};
use silvermere_types::ids::{ActiveMonsterId, SessionId};

use super::{
    check_kill_monster, monster, monster_mut, monster_side, player, player_side, room_of,
    user_ability, user_group,
};
use crate::tell::{Noise, TellBlind, tell_room, tell_user};
use crate::world::World;

/// The kind of companion a character has bonded, if any.
pub fn pet_kind(world: &World, id: SessionId) -> Option<PetKind> {
    let plr = player(world, id)?;
    let n = user_ability(world, plr, fork::PET_KIND);
    let n = u16::try_from(n).ok().filter(|&n| n > 0)?;
    world.pet_kinds.get(&n).cloned()
}

/// How the companion is named to its owner, and to everyone else: its name, or "your
/// hunting hound" / "Devin's hunting hound" until it has one.
pub fn pet_names(world: &World, id: SessionId, kind: &PetKind) -> (String, String) {
    if let Some(name) = world.pet_names.get(&id).filter(|n| !n.is_empty()) {
        return (name.clone(), name.clone());
    }
    let owner = player(world, id)
        .map(|p| p.firstname.clone())
        .unwrap_or_default();
    (
        format!("Your {}", kind.name),
        format!("{owner}'s {}", kind.name),
    )
}

/// What follows a character's name where its companion is shown with it: " and Fang", or
/// " and his hunting hound" until it's named. `None` without a companion.
pub fn shown_suffix(world: &World, id: SessionId) -> Option<String> {
    let kind = pet_kind(world, id)?;
    Some(match world.pet_names.get(&id).filter(|n| !n.is_empty()) {
        Some(name) => format!(" and {name}"),
        None => {
            let his = super::possessive_pronoun(player(world, id));
            format!(" and {his} {}", kind.name)
        }
    })
}

/// `base` (a name as a line shows it) with the companion after it: "Devin and Fang".
pub fn shown_name(world: &World, id: SessionId, base: &str) -> String {
    match shown_suffix(world, id) {
        Some(suffix) => format!("{}{suffix}", base.trim_end()),
        None => base.to_owned(),
    }
}

/// [`shown_name`] for the player in the game whose first name is `firstname`.
pub fn shown_name_of(world: &World, firstname: &str, base: &str) -> String {
    let id = world.session_ids().into_iter().find(|&s| {
        world.is_in_game(s)
            && world
                .session(s)
                .and_then(|x| x.player())
                .is_some_and(|p| p.firstname == firstname)
    });
    match id {
        Some(id) => shown_name(world, id, base),
        None => base.to_owned(),
    }
}

/// The companion's blows at `mon` at the start of its owner's round, if the owner has one
/// and the fight is on.
pub fn pet_round(world: &mut World, id: SessionId, mon: ActiveMonsterId) {
    let Some(kind) = pet_kind(world, id) else {
        return;
    };
    let Some(plr) = player(world, id).cloned() else {
        return;
    };
    let Some(m) = monster(world, mon).cloned() else {
        world.pet_energy.remove(&id);
        return;
    };
    if plr.hp <= 0 || plr.room != m.room {
        return;
    }
    let Some((mut atk, _)) = player_side(
        world,
        id,
        &plr,
        Move::Attack,
        Move::Attack,
        AlignGroup::Neutral,
    ) else {
        return;
    };
    let Some((def, _)) = monster_side(world, &m, 0, Move::Attack, user_group(&plr)) else {
        return;
    };
    let (min, max) = kind.damage(plr.level);
    atk.av += i32::from(kind.accuracy);
    atk.mindmg = min;
    atk.maxdmg = max;
    let cost = i32::from(kind.energy_per_blow.max(1));
    let mut energy = world.pet_energy.get(&id).copied().unwrap_or(0) + TICK_ENERGY;
    let (to_owner, to_room) = pet_names(world, id, &kind);
    let room = room_of(&plr);
    let mut blows = 0;
    while energy >= cost && blows < MAX_BLOWS {
        energy -= cost;
        blows += 1;
        let Some(m) = monster(world, mon).cloned() else {
            break;
        };
        let res = resolve(&mut atk, &def, Move::Attack, &mut world.rng);
        let (mine, theirs) = match res.outcome {
            Outcome::Hit { damage, .. } => {
                let damage = damage.min(i32::from(m.hp)).max(0);
                if let Some(m) = monster_mut(world, mon) {
                    m.hp -= damage as i16;
                    if kind.poison > m.poison {
                        m.poison = kind.poison;
                    }
                }
                world.active_monsters.mark_dirty(mon);
                let hit = |who: &str| Msg::CombatRoomHit {
                    who: who.to_owned(),
                    verb: kind.hit_verb.clone(),
                    target: m.name.clone(),
                    dmg: damage.to_string(),
                };
                (hit(&to_owner), hit(&to_room))
            }
            _ => {
                let miss = |who: &str| Msg::CombatRoomMiss {
                    who: who.to_owned(),
                    verb: kind.miss_verb.clone(),
                    target: m.name.clone(),
                };
                (miss(&to_owner), miss(&to_room))
            }
        };
        tell_user(world, id, mine);
        tell_room(
            world,
            room,
            [Some(id), None],
            Noise::Medium,
            TellBlind::All,
            false,
            None,
            &theirs,
        );
        if check_kill_monster(world, mon, Some(id)) {
            world.pet_energy.remove(&id);
            return;
        }
    }
    world.pet_energy.insert(id, energy.min(i32::from(u16::MAX)));
}
