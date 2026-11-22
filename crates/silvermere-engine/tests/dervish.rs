//! Fork: the Dervish (design/dervish.md): OFFHAND, the two hands' round, and the off-hand
//! weapon moving to an emptied main hand. On the Spellstrike tests' world.

use crate::spellstrike::{caster, lines, plr, spawn, world};
use silvermere_engine::SessionId;
use silvermere_engine::combat::AutoTarget;
use silvermere_engine::combat::autocombat::{do_autocombat_for_user, engage};
use silvermere_engine::commands::input::execute_input;
use silvermere_engine::world::{World, blank};
use silvermere_rules::combat::Move;
use silvermere_types::enums::{ArmourType, WeaponClass, WeaponType};
use silvermere_types::fork;
use silvermere_types::ids::{ClassNo, ItemNo};

const DERVISH: ClassNo = ClassNo(20);
const PLAIN: ClassNo = ClassNo(12);
const SWORD: ItemNo = ItemNo(1);
const DAGGER: ItemNo = ItemNo(901);
const AXE: ItemNo = ItemNo(902);

fn dervish_world() -> World {
    let mut w = world();
    let mut c = blank::clear_class(DERVISH);
    c.name = "Dervish".into();
    c.combat = 7;
    c.weapon = WeaponClass::AnyOneHand;
    c.armour = ArmourType::StuddedRigidLeather;
    c.abilities[0] = (fork::DUAL_WIELD, 10);
    w.classes.insert(DERVISH, c);
    let sword = w.items.get(&SWORD).unwrap().clone();
    let mut dagger = sword.clone();
    dagger.number = DAGGER;
    dagger.name = "dagger".into();
    dagger.speed = 900;
    w.items.insert(DAGGER, dagger);
    let mut axe = sword;
    axe.number = AXE;
    axe.name = "greataxe".into();
    axe.wpntype = WeaponType::TwoHandSharp;
    w.items.insert(AXE, axe);
    w
}

/// A character of `class` carrying the longsword (wielded), a dagger and a greataxe.
fn fighter(w: &mut World, class: ClassNo) -> SessionId {
    let id = caster(w, class, true);
    let p = plr(w, id);
    p.items[0] = SWORD;
    p.items[1] = DAGGER;
    p.items[2] = AXE;
    id
}

fn say(w: &mut World, id: SessionId, line: &str) -> Vec<String> {
    lines(w, id);
    execute_input(w, id, line, false);
    lines(w, id)
}

#[test]
fn offhand_puts_a_second_weapon_in_the_off_hand() {
    let mut w = dervish_world();
    let id = fighter(&mut w, DERVISH);
    let out = say(&mut w, id, "offhand dagger");
    assert!(
        out.iter()
            .any(|l| l == "You are now holding dagger in your off-hand."),
        "{out:?}"
    );
    assert!(plr(&mut w, id).worn.contains(&DAGGER));
    let out = say(&mut w, id, "i");
    assert!(out.iter().any(|l| l.contains("dagger (Off-Hand)")), "{out:?}");
    let out = say(&mut w, id, "offhand dagger");
    assert!(
        out.iter()
            .any(|l| l == "You already hold a weapon in your off-hand."),
        "{out:?}"
    );
}

#[test]
fn offhand_refuses_what_a_dervish_cannot_hold() {
    let mut w = dervish_world();
    let id = fighter(&mut w, DERVISH);
    let out = say(&mut w, id, "offhand greataxe");
    assert!(
        out.iter()
            .any(|l| l == "You can only hold a one-handed melee weapon in your off-hand."
                || l == "You may not use that weapon."),
        "{out:?}"
    );
    // the main hand's only longsword can't be held twice
    let out = say(&mut w, id, "offhand longsword");
    assert!(
        out.iter().any(|l| l == "You have no other longsword to hold."),
        "{out:?}"
    );
    // nothing in the main hand
    plr(&mut w, id).weapon = ItemNo(0);
    let out = say(&mut w, id, "offhand dagger");
    assert!(
        out.iter()
            .any(|l| l == "You must wield a weapon in your main hand first."),
        "{out:?}"
    );
    assert!(!plr(&mut w, id).worn.contains(&DAGGER));
}

#[test]
fn offhand_is_no_command_for_other_classes() {
    let mut w = dervish_world();
    let id = fighter(&mut w, PLAIN);
    let out = say(&mut w, id, "offhand dagger");
    assert!(!out.iter().any(|l| l.contains("off-hand")), "{out:?}");
    assert!(!plr(&mut w, id).worn.contains(&DAGGER));
}

#[test]
fn a_dervish_uses_no_ranged_weapon() {
    let mut w = dervish_world();
    let sling = w.items.get_mut(&DAGGER).unwrap();
    sling.abilities[0] = (fork::RANGED, 0);
    let sling = sling.clone();
    let id = fighter(&mut w, DERVISH);
    let p = plr(&mut w, id).clone();
    assert!(!silvermere_engine::items::equip::user_can_use(&w, &p, &sling));
    let id = fighter(&mut w, PLAIN);
    let p = plr(&mut w, id).clone();
    assert!(silvermere_engine::items::equip::user_can_use(&w, &p, &sling));
}

/// The blows of one round against the dummy, with or without the dagger in the off-hand.
fn blows(level: u16, agility: u16, speed: i16, offhand: bool) -> (usize, Vec<String>) {
    let mut w = dervish_world();
    for n in [SWORD, DAGGER] {
        w.items.get_mut(&n).unwrap().speed = speed;
    }
    let id = fighter(&mut w, DERVISH);
    {
        let p = plr(&mut w, id);
        p.level = level;
        p.agi = agility;
    }
    if offhand {
        say(&mut w, id, "offhand dagger");
    }
    let m = spawn(&mut w);
    engage(&mut w, id, Some(AutoTarget::Monster(m)), Move::Attack, None);
    lines(&mut w, id);
    plr(&mut w, id).energy = 1000;
    do_autocombat_for_user(&mut w, id);
    let out = lines(&mut w, id);
    let n = out.iter().filter(|l| l.contains("practice dummy")).count();
    (n, out)
}

#[test]
fn each_hand_strikes_at_least_once_a_round() {
    // weapons far too slow for a round's energy: still a blow with each hand
    let (one, out) = blows(1, 50, 5000, false);
    assert_eq!(one, 1, "one hand, the minimum: {out:?}");
    let (two, out) = blows(1, 50, 5000, true);
    assert_eq!(two, 2, "two hands, a blow each: {out:?}");
}

#[test]
fn two_hands_make_at_most_eight_blows() {
    let (n, out) = blows(60, 200, 200, true);
    assert_eq!(n, 8, "{out:?}");
}

#[test]
fn the_off_hand_adds_blows_to_a_round() {
    let (one, _) = blows(10, 50, 1000, false);
    let (two, out) = blows(10, 50, 1000, true);
    assert!(two > one, "{one} with one hand, {two} with two: {out:?}");
}

#[test]
fn an_emptied_main_hand_takes_the_off_hand_weapon() {
    let mut w = dervish_world();
    let id = fighter(&mut w, DERVISH);
    say(&mut w, id, "offhand dagger");
    let out = say(&mut w, id, "remove longsword");
    assert!(
        out.iter().any(|l| l == "You shift dagger to your main hand."),
        "{out:?}"
    );
    let p = plr(&mut w, id).clone();
    assert_eq!(p.weapon, DAGGER);
    assert!(!p.worn.contains(&DAGGER));
}

#[test]
fn others_see_the_off_hand_weapon_worn_off_hand() {
    let mut w = dervish_world();
    let id = fighter(&mut w, DERVISH);
    say(&mut w, id, "offhand dagger");
    let bob = crate::common::enter(
        &mut w,
        "Bob",
        crate::spellstrike::ROOM,
        silvermere_types::enums::MessageLevel::Low,
    );
    plr(&mut w, bob).settings = silvermere_types::flags::PlayerSettings::VALIDATED;
    let out = say(&mut w, bob, "look gorin");
    let line = out.iter().find(|l| l.contains("dagger")).cloned();
    assert!(
        line.as_deref().is_some_and(|l| l.contains("(Off-Hand)") && !l.contains("Nowhere")),
        "{out:?}"
    );
}
