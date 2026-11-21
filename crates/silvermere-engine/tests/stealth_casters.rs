//! Fork: the Avenger's spell backstab, VANISH and blunt backstab
//! (design/stealth-casters.md), on the Spellstrike tests' world.

use crate::spellstrike::{caster, lines, plr, spawn, world};
use silvermere_engine::SessionId;
use silvermere_engine::combat::AutoTarget;
use silvermere_engine::combat::autocombat::{do_autocombat_for_user, engage};
use silvermere_engine::commands::input::execute_input;
use silvermere_engine::world::{World, blank};
use silvermere_rules::combat::Move;
use silvermere_types::enums::{Ability, ArmourType, Magery, WeaponClass, WeaponType};
use silvermere_types::fork;
use silvermere_types::ids::{ActiveMonsterId, ClassNo, ItemNo, SpellNo};

const AVENGER: ClassNo = ClassNo(18);
const PLAIN: ClassNo = ClassNo(12);
const MMIS: SpellNo = SpellNo(1);
const DUMMY_HP: i16 = 9999;

/// An Avenger-like class (Mage magery, to cast the test world's mage spells) whose stealth
/// rolls are as good or as bad as `stealth_penalty` makes them.
fn avenger(w: &mut World, stealth_penalty: i16, vanish_penalty: i16) {
    let mut c = blank::clear_class(AVENGER);
    c.name = "Avenger".into();
    c.magery = Magery::Mage;
    c.magelvl = 1;
    c.combat = 5;
    c.weapon = WeaponClass::AnyBlunt;
    c.armour = ArmourType::StuddedRigidLeather;
    c.abilities[0] = (Ability::ClassStealth, 0);
    c.abilities[1] = (Ability::BackstabMin, 5);
    c.abilities[2] = (Ability::BackstabMax, 5);
    c.abilities[3] = (fork::SPELL_BACKSTAB, 100);
    c.abilities[4] = (fork::SPELL_BACKSTAB_STEALTH, stealth_penalty);
    c.abilities[5] = (fork::VANISH, vanish_penalty);
    c.abilities[6] = (fork::BLUNT_BACKSTAB, 0);
    w.classes.insert(AVENGER, c);
}

fn say(w: &mut World, id: SessionId, line: &str) -> Vec<String> {
    lines(w, id);
    execute_input(w, id, line, false);
    lines(w, id)
}

fn round(w: &mut World, id: SessionId) -> Vec<String> {
    plr(w, id).energy = 1000;
    do_autocombat_for_user(w, id);
    lines(w, id)
}

fn dummy_hp(w: &World, m: ActiveMonsterId) -> i16 {
    w.active_monsters
        .values()
        .find(|x| x.number == m)
        .map_or(0, |x| x.hp)
}

/// An Avenger hidden in the room with the dummy, casting magic missile at it with `CAST`,
/// then the first round. Returns the round's lines and the damage done.
fn cast_from_hiding(stealth_penalty: i16, hidden: bool) -> (Vec<String>, Vec<String>, i16) {
    let mut w = world();
    avenger(&mut w, stealth_penalty, 20);
    // a spell of fixed strength, so a backstab's damage is plain to see
    let spl = w.spells.get_mut(&MMIS).unwrap();
    (spl.minval, spl.maxval) = (10, 10);
    let id = caster(&mut w, AVENGER, false);
    plr(&mut w, id).stealth = 90;
    if hidden {
        plr(&mut w, id).hidden = 1;
    }
    let m = spawn(&mut w);
    let begun = say(&mut w, id, "cast magic missile practice dummy");
    assert_eq!(plr(&mut w, id).hidden, 0, "casting reveals the caster");
    let out = round(&mut w, id);
    (begun, out, DUMMY_HP - dummy_hp(&w, m))
}

#[test]
fn a_spell_from_hiding_lands_as_a_backstab() {
    // a penalty far below zero: the stealth roll can't fail
    let (begun, out, dmg) = cast_from_hiding(-200, true);
    assert!(!begun.iter().any(|l| l.contains("spots you")), "{begun:?}");
    assert!(
        out.iter()
            .any(|l| l == "You strike from the shadows with magic missile!"),
        "{out:?}"
    );
    // as a melee backstab: the spell's 10 or more doubled, plus twice the level (10), a
    // tenth of the stealth (90) and the backstab bonus of 5, raised by 10%
    assert!(dmg >= 59, "backstab damage, got {dmg}: {out:?}");
    let (_, _, normal) = cast_from_hiding(-200, false);
    assert!(normal < 25, "a normal cast, got {normal}");
}

#[test]
fn a_spotted_caster_casts_as_normal() {
    let (begun, out, dmg) = cast_from_hiding(200, true);
    assert!(
        begun
            .iter()
            .any(|l| l == "Your foe spots you as you begin to cast!"),
        "{begun:?}"
    );
    assert!(!out.iter().any(|l| l.contains("from the shadows")), "{out:?}");
    assert!(dmg > 0 && dmg < 25, "normal damage, got {dmg}");
}

#[test]
fn a_spell_in_the_open_is_no_backstab() {
    let (begun, out, dmg) = cast_from_hiding(-200, false);
    assert!(!begun.iter().any(|l| l.contains("spots you")), "{begun:?}");
    assert!(!out.iter().any(|l| l.contains("from the shadows")), "{out:?}");
    assert!(dmg > 0 && dmg < 25, "normal damage, got {dmg}");
}

#[test]
fn only_the_first_cast_is_a_backstab() {
    let mut w = world();
    avenger(&mut w, -200, 20);
    let id = caster(&mut w, AVENGER, false);
    plr(&mut w, id).hidden = 1;
    spawn(&mut w);
    say(&mut w, id, "cast magic missile practice dummy");
    let first = round(&mut w, id);
    let second = round(&mut w, id);
    assert!(first.iter().any(|l| l.contains("from the shadows")), "{first:?}");
    assert!(!second.iter().any(|l| l.contains("from the shadows")), "{second:?}");
}

/// A fighter with a mace in the room with the dummy.
fn with_mace(class: ClassNo) -> (World, SessionId) {
    let mut w = world();
    avenger(&mut w, 20, 20);
    let mace = ItemNo(1);
    let item = w.items.get_mut(&mace).unwrap();
    item.name = "mace".into();
    item.wpntype = WeaponType::OneHandBlunt;
    item.abilities.iter_mut().for_each(|s| *s = (Ability::None, 0));
    let id = caster(&mut w, class, true);
    plr(&mut w, id).hidden = 1;
    spawn(&mut w);
    (w, id)
}

#[test]
fn an_avenger_backstabs_with_a_blunt_weapon() {
    let (mut w, id) = with_mace(AVENGER);
    let out = say(&mut w, id, "backstab practice dummy");
    assert!(!out.iter().any(|l| l.contains("cannot backstab")), "{out:?}");
    let out = round(&mut w, id);
    assert!(out.iter().any(|l| l.contains("surprise")), "{out:?}");
    // another class can't
    let (mut w, id) = with_mace(PLAIN);
    let out = say(&mut w, id, "backstab practice dummy");
    assert!(
        out.iter()
            .any(|l| l == "You cannot backstab with this weapon."),
        "{out:?}"
    );
}

/// An Avenger fighting the dummy in melee, its vanish as sure or as hopeless as
/// `vanish_penalty` makes it.
fn fighting(vanish_penalty: i16) -> (World, SessionId) {
    let mut w = world();
    avenger(&mut w, 20, vanish_penalty);
    let id = caster(&mut w, AVENGER, true);
    plr(&mut w, id).stealth = 90;
    let m = spawn(&mut w);
    engage(&mut w, id, Some(AutoTarget::Monster(m)), Move::Attack, None);
    lines(&mut w, id);
    (w, id)
}

#[test]
fn vanish_hides_the_avenger_and_ends_its_fight() {
    let (mut w, id) = fighting(-200);
    let out = say(&mut w, id, "vanish");
    assert!(
        out.iter()
            .any(|l| l == "You slip away and melt into the shadows."),
        "{out:?}"
    );
    assert_eq!(plr(&mut w, id).hidden, 1);
    assert!(!w.autocombat.queued(id), "out of the fight");
}

#[test]
fn a_failed_vanish_takes_the_round_and_waits() {
    let (mut w, id) = fighting(200);
    let out = say(&mut w, id, "vanish");
    assert!(
        out.iter()
            .any(|l| l == "You try to slip away, but your foe keeps you in sight."),
        "{out:?}"
    );
    assert_eq!(plr(&mut w, id).hidden, 0);
    let skipped = round(&mut w, id);
    assert!(
        !skipped.iter().any(|l| l.contains("practice dummy")),
        "no blows this round: {skipped:?}"
    );
    assert!(w.autocombat.queued(id), "still fighting");
    let out = say(&mut w, id, "vanish");
    assert!(
        out.iter().any(|l| l == "You can't vanish again so soon."),
        "{out:?}"
    );
    // five rounds on, it may try again
    for _ in 0..5 {
        silvermere_engine::commands::vanish::round_passed(&mut w);
    }
    let out = say(&mut w, id, "vanish");
    assert!(out.iter().any(|l| l.contains("slip away")), "{out:?}");
}

#[test]
fn vanish_is_for_a_fight_and_for_avengers() {
    let mut w = world();
    avenger(&mut w, 20, -200);
    let id = caster(&mut w, AVENGER, true);
    let out = say(&mut w, id, "vanish");
    assert!(
        out.iter().any(|l| l.contains("only vanish in the middle of a fight")),
        "{out:?}"
    );
    let mut w = world();
    let id = caster(&mut w, PLAIN, true);
    let out = say(&mut w, id, "vanish");
    assert!(!out.iter().any(|l| l.contains("vanish")), "{out:?}");
    assert!(w.vanish_wait.is_empty());
}

#[test]
fn a_spell_backstab_is_built_as_a_melee_backstab() {
    use silvermere_engine::combat::ambush::backstab_amount;
    // a level 50 godstrike of 67 with stealth 164: (134 + 100 + 16 + 5) * 150%
    assert_eq!(backstab_amount(67, 100, 50, 164, 5), 382);
    // a level 1 harm of 13 with stealth 30: (26 + 2 + 3 + 5) * 101%
    assert_eq!(backstab_amount(13, 100, 1, 30, 5), 36);
}
