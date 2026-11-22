//! Fork: OFFHAND (design/dervish.md). A class with `DualWield` puts a second one-handed
//! melee weapon in its off-hand: a worn slot, so the weapon's bonuses count as any worn
//! item's and it is saved with the character. The line is taken only from such a class, so
//! for anyone else OFFHAND stays an unknown command.

use silvermere_text::Msg;
use silvermere_types::enums::{ItemType, WeaponType};
use silvermere_types::flags::PlayerStatus;
use silvermere_types::fork;
use silvermere_types::ids::{ItemNo, SessionId};
use silvermere_types::sizes::NWORN;

use super::parse::{CommandLine, eq_ignore_case};
use crate::combat::dual::{offhand_weapon, tell_room_wields};
use crate::combat::{player, player_mut, possessive_pronoun, user_has};
use crate::items::equip;
use crate::items::inventory::{Search, find_item_in_inventory};
use crate::tell::tell_user;
use crate::world::World;

/// Handles `line` when it is OFFHAND from a class that can dual-wield.
pub fn try_offhand(world: &mut World, id: SessionId, line: &CommandLine) -> bool {
    if !eq_ignore_case(line.first(), "offhand") {
        return false;
    }
    let Some(plr) = player(world, id) else {
        return false;
    };
    if !user_has(world, plr, fork::DUAL_WIELD) {
        return false;
    }
    offhand(world, id, line);
    true
}

fn offhand(world: &mut World, id: SessionId, line: &CommandLine) {
    if line.argc() == 1 {
        tell_user(world, id, Msg::OffhandSyntax);
        return;
    }
    let Some(plr) = player(world, id) else {
        return;
    };
    if plr.weapon.0 == 0 {
        tell_user(world, id, Msg::OffhandNeedMain);
        return;
    }
    if offhand_weapon(world, plr).is_some() {
        tell_user(world, id, Msg::OffhandFull);
        return;
    }
    let name = line.rest_from(1).to_owned();
    let f = find_item_in_inventory(world, plr, &name, Search::Any);
    let Some(itm) = f.item.and_then(|n| world.items.get(&n)).cloned() else {
        tell_user(world, id, Msg::NotHaveA { name });
        return;
    };
    if itm.item_type != ItemType::Weapon {
        tell_user(world, id, Msg::NotAWeapon { item: itm.name });
        return;
    }
    // a copy not already in use: the main hand's weapon may be a second of the same
    let carried = plr.items.iter().filter(|&&n| n == itm.number).count();
    let in_use = usize::from(plr.weapon == itm.number)
        + plr.worn.iter().filter(|&&n| n == itm.number).count();
    if carried <= in_use {
        tell_user(world, id, Msg::OffhandInMain { item: itm.name });
        return;
    }
    if !equip::user_can_use(world, plr, &itm) {
        tell_user(world, id, Msg::WeaponForbidden);
        return;
    }
    let two_handed = matches!(
        itm.wpntype,
        WeaponType::TwoHandBlunt | WeaponType::TwoHandSharp
    );
    if two_handed || silvermere_rules::abilities::item_has_ability(&itm, fork::RANGED) {
        tell_user(world, id, Msg::OffhandNotThat);
        return;
    }
    let Some(slot) = plr.worn.iter().take(NWORN).position(|&w| w == ItemNo(0)) else {
        tell_user(world, id, Msg::WearNoRoom);
        return;
    };
    let his = possessive_pronoun(Some(plr)).to_owned();
    // changing equipment breaks off auto-combat, and gives the character away
    equip::break_off_autocombat(world, id, true);
    let Some(p) = player_mut(world, id) else {
        return;
    };
    p.status.remove(PlayerStatus::SNEAKING);
    p.hidden = 0;
    p.worn[slot] = itm.number;
    p.wornuses[slot] = f.charges;
    equip::equip_apply_item_abilities(world, id, &itm);
    world.offhand_energy.remove(&id);
    tell_room_wields(world, id, &itm.name, &his);
    tell_user(world, id, Msg::OffhandOk { item: itm.name });
    equip::recalc(world, id);
}
