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
fn a_hound_strikes_once_a_round_before_its_owner() {
    let (mut w, id, _) = fight(1);
    let out = round(&mut w, id);
    assert_eq!(pet_lines(&out, "hunting hound"), 1, "{out:?}");
    let first = out.first().unwrap();
    assert!(
        first.starts_with("Your hunting hound "),
        "before the owner's blows: {out:?}"
    );
    assert!(
        first.contains("bites practice dummy for") || first.contains("snaps at practice dummy"),
        "{out:?}"
    );
    assert!(out.len() > 1, "the owner still strikes: {out:?}");
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

fn say(w: &mut World, id: SessionId, line: &str) -> Vec<String> {
    lines(w, id);
    silvermere_engine::commands::input::execute_input(w, id, line, false);
    lines(w, id)
}

#[test]
fn pet_shows_and_pet_name_names_the_companion() {
    let (mut w, id, _) = fight(1);
    w.session_mut(id).unwrap().account = Some(silvermere_engine::player_session::AccountRef {
        id: silvermere_types::ids::AccountId(7),
        username: "gorin".into(),
        sysop: false,
        gameop: false,
        sex: b'M',
    });
    let out = say(&mut w, id, "pet");
    assert!(
        out.iter()
            .any(|l| l.starts_with("Your hunting hound fights at your side")),
        "{out:?}"
    );
    let out = say(&mut w, id, "pet name fANG");
    assert_eq!(out[0], "Your hunting hound is now called Fang.", "{out:?}");
    assert_eq!(w.pet_names.get(&id).map(String::as_str), Some("Fang"));
    let saved = w.queued_outputs().iter().any(|o| {
        matches!(o, silvermere_engine::EngineOutput::Persist(
            silvermere_engine::PersistRequest::Pet(p)) if p.name == "Fang" && p.kind == 1)
    });
    assert!(saved, "the name is saved");
    let out = say(&mut w, id, "pet name Fang2");
    assert_eq!(
        out[0],
        "A companion's name is one word of letters, up to 10 long."
    );
    assert_eq!(w.pet_names.get(&id).map(String::as_str), Some("Fang"));
}

#[test]
fn pet_is_left_alone_without_a_companion() {
    let (mut w, id, _) = fight(0);
    let out = say(&mut w, id, "pet name Fang");
    assert!(!out.iter().any(|l| l.contains("now called")), "{out:?}");
    assert!(w.pet_names.is_empty());
}

#[test]
fn others_see_the_warden_with_the_companion() {
    let (mut w, id, _) = fight(1);
    w.pet_names.insert(id, "Fang".into());
    let bob = crate::common::enter(
        &mut w,
        "Bob",
        crate::spellstrike::ROOM,
        silvermere_types::enums::MessageLevel::Low,
    );
    plr(&mut w, bob).settings = silvermere_types::flags::PlayerSettings::VALIDATED;
    let out = say(&mut w, bob, "look");
    assert!(
        out.iter()
            .any(|l| l.contains("Also here:") && l.contains("Gorin and Fang")),
        "{out:?}"
    );
    let out = say(&mut w, bob, "who");
    assert!(out.iter().any(|l| l.contains("and Fang")), "{out:?}");
    // unnamed, it goes by its kind
    w.pet_names.remove(&id);
    let out = say(&mut w, bob, "look");
    assert!(
        out.iter()
            .any(|l| l.contains("Gorin and his hunting hound")),
        "{out:?}"
    );
}

#[test]
fn stable_script_patterns_match() {
    use silvermere_engine::scripts::wildcard_match;
    assert!(wildcard_match(
        "buy ha*wkling's leash",
        "buy hawkling's leash"
    ));
    assert!(wildcard_match("buy ho*und's leash", "buy hound"));
    assert!(!wildcard_match(
        "buy ho*und's leash",
        "buy hawkling's leash"
    ));
}

#[test]
fn stable_script_lines_run() {
    use silvermere_engine::scripts::{LineResult, pattern_lines, perform_matched_action};
    let (mut w, id, _) = fight(0);
    let text = "buy ho*und's leash:failability 1012:giveability 1012 1\nbuy ho*und's leash:takeitem 9000 0:text 20037\n";
    let lines = pattern_lines(text);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_eq!(lines[0].pattern, "buy ho*und's leash");
    let r = perform_matched_action(&mut w, id, &lines[0].action);
    assert_eq!(r, LineResult::Ok, "{:?}", lines[0].action);
    let has = plr(&mut w, id)
        .abils
        .iter()
        .any(|s| s.0 == fork::STARTER_LEASH && s.1 == 1);
    assert!(has);
    let r = perform_matched_action(&mut w, id, &lines[0].action);
    assert_eq!(r, LineResult::Stop, "already marked");
}
