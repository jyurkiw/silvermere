//! Fork: VANISH (design/stealth-casters.md). A class with the `Vanish` ability can slip
//! back into hiding in the middle of a fight. The roll is the stock hide chance less the
//! ability's value; it takes the round either way, and can be tried once every
//! [`VANISH_ROUNDS`] rounds. On success the character's auto-combat ends and it is hidden,
//! so monsters (which pick their foes each turn among the players they can see) lose it.
//! The line is taken only from a class that has the ability, so for anyone else VANISH
//! stays an unknown command.

use silvermere_rules::Rng;
use silvermere_text::Msg;
use silvermere_types::fork;
use silvermere_types::flags::{PlayerStatus, PlayerStatus2};
use silvermere_types::ids::SessionId;

use super::parse::{CommandLine, eq_ignore_case};
use crate::combat::autocombat::{
    display_autocombat_broken, is_being_attacked, is_inside_autocombat, kill_autocombat,
    kill_autocombat_against,
};
use crate::combat::{player, player_mut, room_of, user_ability, user_has};
use crate::items::equip;
use crate::movement::move_user::calc_hide_sneak_chance;
use crate::tell::{Noise, TellBlind, tell_room, tell_user};
use crate::world::World;

/// The rounds between one VANISH and the next.
pub const VANISH_ROUNDS: u8 = 5;

/// Handles `line` when it is VANISH from a class that can vanish.
pub fn try_vanish(world: &mut World, id: SessionId, line: &CommandLine) -> bool {
    if line.argc() != 1 || !eq_ignore_case(line.first(), "vanish") {
        return false;
    }
    let Some(plr) = player(world, id) else {
        return false;
    };
    if !user_has(world, plr, fork::VANISH) {
        return false;
    }
    let fighting =
        is_inside_autocombat(world, id) || is_being_attacked(world, id) || plr.attackers > 0;
    let status2 = plr.status2;
    let penalty = user_ability(world, plr, fork::VANISH);
    if !fighting {
        tell_user(world, id, Msg::VanishNotFighting);
        return true;
    }
    if status2.contains(PlayerStatus2::HELD) {
        tell_user(world, id, Msg::HideHeld);
        return true;
    }
    if status2.contains(PlayerStatus2::KNOCKEDDOWN) {
        tell_user(world, id, Msg::HideStunned);
        return true;
    }
    if world.vanish_wait.get(&id).is_some_and(|&n| n > 0) {
        tell_user(world, id, Msg::VanishWait);
        return true;
    }
    world.vanish_wait.insert(id, VANISH_ROUNDS);
    let chance = (calc_hide_sneak_chance(world, id, 95, Some(id)) - penalty).clamp(0, 95);
    if world.rng.range(0, 100) < chance {
        if is_inside_autocombat(world, id) {
            display_autocombat_broken(world, id, false);
        }
        kill_autocombat(world, id);
        // players fighting it lose it too
        kill_autocombat_against(world, id);
        world.vanish_skip.remove(&id);
        let Some(p) = player_mut(world, id) else {
            return true;
        };
        p.status.remove(PlayerStatus::RESTING);
        p.hidden = 1;
        let who = p.firstname.clone();
        let room = room_of(p);
        tell_user(world, id, Msg::VanishDone);
        tell_room(
            world,
            room,
            [Some(id), None],
            Noise::Always,
            TellBlind::All,
            false,
            None,
            &Msg::VanishDoneRoom { who },
        );
    } else {
        tell_user(world, id, Msg::VanishFailed);
        // the attempt takes the next round's blows
        world.vanish_skip.insert(id);
    }
    equip::recalc(world, id);
    true
}

/// A round has passed: every character's wait for its next VANISH shortens.
pub fn round_passed(world: &mut World) {
    world.vanish_wait.retain(|_, n| {
        *n = n.saturating_sub(1);
        *n > 0
    });
}
