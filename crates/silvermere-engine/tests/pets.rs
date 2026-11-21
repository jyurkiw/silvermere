//! Fork: a companion's blows after its owner's round (design/warden.md).

use crate::spellstrike::{WAR_MAGE, caster, lines, plr, spawn, world};
use silvermere_engine::SessionId;
use silvermere_engine::combat::AutoTarget;
use silvermere_engine::combat::autocombat::{do_autocombat_for_user, engage};
use silvermere_engine::world::World;
use silvermere_rules::combat::Move;
use silvermere_types::fork::{self, PetKind};
use silvermere_types::ids::ActiveMonsterId;

fn kind(number: u16, name: &str, energy_per_blow: u16, poison: i16) -> PetKind {
    PetKind {
        number,
        name: name.into(),
        hit_verb: "bites".into(),
        miss_verb: "snaps at".into(),
        energy_per_blow,
        min_base: 2,
        min_per_level: 50,
        max_base: 8,
        max_per_level: 200,
        accuracy: 50,
        poison,
    }
}

/// A fighter with companion `n` bonded, fighting the dummy in melee.
fn fight(n: u16) -> (World, SessionId, ActiveMonsterId) {
    let mut w = world();
    for k in [
        kind(1, "hunting hound", 1000, 0),
        kind(2, "hawkling", 333, 0),
        kind(3, "trained rattlesnake", 1000, 5),
        kind(4, "drop bear", 2000, 0),
    ] {
        w.pet_kinds.insert(k.number, k);
    }
    let id = caster(&mut w, WAR_MAGE, true);
    plr(&mut w, id).abils[0] = (fork::PET_KIND, n as i16);
    let m = spawn(&mut w);
    engage(&mut w, id, Some(AutoTarget::Monster(m)), Move::Attack, None);
    lines(&mut w, id);
    (w, id, m)
}

fn round(w: &mut World, id: SessionId) -> Vec<String> {
    plr(w, id).energy = 1000;
    do_autocombat_for_user(w, id);
    lines(w, id)
}

fn pet_lines(out: &[String], name: &str) -> usize {
    out.iter()
        .filter(|l| l.starts_with(&format!("Your {name} ")))
        .count()
}

#[test]
fn a_hound_strikes_once_a_round_after_its_owner() {
    let (mut w, id, _) = fight(1);
    let out = round(&mut w, id);
    assert_eq!(pet_lines(&out, "hunting hound"), 1, "{out:?}");
    let last = out.last().unwrap();
    assert!(
        last.starts_with("Your hunting hound "),
        "after the owner's blows: {out:?}"
    );
    assert!(
        last.contains("bites practice dummy for") || last.contains("snaps at practice dummy"),
        "{out:?}"
    );
}

#[test]
fn a_hawkling_strikes_three_times_and_a_drop_bear_every_other_round() {
    let (mut w, id, _) = fight(2);
    assert_eq!(pet_lines(&round(&mut w, id), "hawkling"), 3);
    let (mut w, id, _) = fight(4);
    let first = pet_lines(&round(&mut w, id), "drop bear");
    let second = pet_lines(&round(&mut w, id), "drop bear");
    assert_eq!((first, second), (0, 1), "a blow every other round");
}

#[test]
fn a_rattlesnake_bite_poisons() {
    let (mut w, id, m) = fight(3);
    for _ in 0..5 {
        round(&mut w, id);
    }
    let mon = w.active_monsters.values().find(|x| x.number == m).unwrap();
    assert!(mon.poison >= 5, "a landed bite leaves poison");
}

#[test]
fn no_companion_no_extra_blows() {
    let (mut w, id, _) = fight(0);
    let out = round(&mut w, id);
    assert!(!out.iter().any(|l| l.starts_with("Your ")), "{out:?}");
}

#[test]
fn a_named_companion_goes_by_its_name() {
    let (mut w, id, _) = fight(1);
    w.pet_names.insert(id, "Fang".into());
    let out = round(&mut w, id);
    assert!(out.iter().any(|l| l.starts_with("Fang ")), "{out:?}");
}
