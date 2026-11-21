//! Fork: the Spellstrike round (design/war-mage.md). A War Mage-like class with a weapon
//! and an attack spell casts first, at a discounted blow's energy, then strikes with what
//! the round has left. Anything else stays the stock casting round.

use crate::common;

use common::{enter, test_config, tiny_snapshot};
use silvermere_engine::SessionId;
use silvermere_engine::combat::autocombat::{do_autocombat_for_user, engage};
use silvermere_engine::combat::spellstrike::is_spellstrike_spell;
use silvermere_engine::combat::{AutoTarget, player_side};
use silvermere_engine::world::{World, blank};
use silvermere_rules::SeededRng;
use silvermere_rules::combat::{AlignGroup, Move};
use silvermere_text::strip_ansi;
use silvermere_types::enums::{
    Ability, Alignment, ArmourType, Element, GroupRole, ItemType, Magery, MessageLevel,
    MonsterType, ResistMode, ResistStat, SpellTarget, SpellType, WeaponClass, WeaponType,
};
use silvermere_types::flags::{ActiveMonsterFlags, PlayerSettings, SpellFlags};
use silvermere_types::fork;
use silvermere_types::ids::{
    ActiveMonsterId, ClassNo, ItemNo, MapId, MessageNo, MonsterNo, RaceNo, RoomId, RoomKey, SpellNo,
};
use silvermere_types::records::{ActiveMonster, ActiveSpell, Character, Coins, Spell};

const ROOM: u32 = 2140;
pub(crate) const WAR_MAGE: ClassNo = ClassNo(16);
const PLAIN_MAGE: ClassNo = ClassNo(12);
const HUMAN: RaceNo = RaceNo(1);
const MMIS: SpellNo = SpellNo(1);
const BALL: SpellNo = SpellNo(2);
const DUMMY: MonsterNo = MonsterNo(200);
const SWORD: ItemNo = ItemNo(1);

fn spell(n: i16, name: &str, target: SpellTarget) -> Spell {
    let mut a: [(Ability, i16); 10] = std::array::from_fn(|_| (Ability::None, 0));
    a[0] = (Ability::DamageArmour, 0);
    Spell {
        number: SpellNo(n),
        name: name.into(),
        descs: [String::new(), String::new()],
        resist: 0,
        levelcap: 0,
        flags: SpellFlags::empty(),
        abilities: a,
        energy: 1000,
        level: 1,
        minval: 4,
        maxval: 12,
        spell_type: SpellType::Autocombat,
        resmode: ResistMode::from_code(0),
        diff: 15,
        durlvl: 0,
        target,
        duration: 0,
        element: Element::from_code(4),
        resstat: ResistStat::from_code(0),
        magery: Magery::Mage,
        msgnum: MessageNo(0),
        mana: 4,
        maxinc: 0,
        maxlvls: 0,
        magelvl: 1,
        mininc: 0,
        minlvls: 0,
        durinc: 0,
        durlvls: 0,
        shortnm: name[..4].into(),
    }
}

fn class(n: ClassNo, spellstrike: bool) -> silvermere_types::records::Class {
    let mut c = blank::clear_class(n);
    c.name = if spellstrike { "War Mage" } else { "Mage" }.into();
    c.magery = Magery::Mage;
    c.magelvl = 1;
    c.combat = 6;
    c.weapon = WeaponClass::All;
    c.armour = ArmourType::Chainmail;
    if spellstrike {
        c.abilities[0] = (fork::SPELLSTRIKE, 30);
        c.abilities[1] = (fork::SPELLSTRIKE_SUCCESS_OFF, 10);
        c.abilities[2] = (fork::SPELLSTRIKE_FAIL_OFF, 40);
        c.abilities[3] = (fork::NO_OFF_HAND, 0);
    }
    c
}

pub(crate) fn world() -> World {
    let mut s = tiny_snapshot();
    s.spells = vec![
        spell(1, "magic missile", SpellTarget::MonsterOrSingle),
        spell(2, "fireball", SpellTarget::FullAttack),
    ];
    let mut k = blank::clear_known_monster(DUMMY);
    k.name = "practice dummy".into();
    k.level = 1;
    k.maxhp = 9999;
    k.energy = 1000;
    k.alignment = Alignment::Neutral;
    k.montype = MonsterType::Lair;
    s.monsters.push(k);
    let sword = s.items.iter_mut().find(|i| i.number == SWORD).unwrap();
    sword.name = "longsword".into();
    sword.item_type = ItemType::Weapon;
    sword.wpntype = WeaponType::OneHandSharp;
    sword.speed = 1350;
    sword.mindmg = 3;
    sword.maxdmg = 12;
    sword.weight = 1;
    s.classes.push(class(WAR_MAGE, true));
    s.classes.push(class(PLAIN_MAGE, false));
    let mut human = blank::clear_race(HUMAN);
    human.name = "Human".into();
    s.races.retain(|r| r.number != HUMAN);
    s.races.push(human);
    let mut w = World::load(test_config(), 8, s).expect("world loads");
    w.rng = SeededRng::new(7);
    w
}

pub(crate) fn plr(w: &mut World, id: SessionId) -> &mut Character {
    w.session_mut(id).unwrap().character.as_mut().unwrap()
}

/// A level 10 caster of `class` in the room, armed unless `armed` is false.
pub(crate) fn caster(w: &mut World, class: ClassNo, armed: bool) -> SessionId {
    let id = enter(w, "Gorin", ROOM, MessageLevel::Low);
    let c = plr(w, id);
    c.settings = PlayerSettings::VALIDATED | PlayerSettings::CANCAST;
    c.class = class;
    c.race = HUMAN;
    c.level = 10;
    c.sex = b'M';
    c.agi = 50;
    c.str = 60;
    (c.hp, c.maxhp) = (500, 500);
    (c.mana, c.maxmana) = (40, 40);
    (c.energy, c.maxenergy) = (1000, 1000);
    c.spellcast = 300;
    c.nspellslots = 100;
    c.spellbook = vec![SpellNo(0); 100];
    c.spellbook[0] = MMIS;
    c.spellbook[1] = BALL;
    if armed {
        c.weapon = SWORD;
    }
    id
}

pub(crate) fn spawn(w: &mut World) -> ActiveMonsterId {
    let k = w.monsters.get(&DUMMY).unwrap().clone();
    let number = ActiveMonsterId(1);
    w.active_monsters.insert(ActiveMonster {
        number,
        knownnum: k.number,
        level: k.level,
        map: MapId(1),
        room: RoomId(ROOM),
        poison: 0,
        energy: k.energy,
        hp: k.maxhp,
        following: String::new(),
        trail: std::array::from_fn(|_| RoomId::default()),
        hunters: std::array::from_fn(|_| ActiveMonsterId::default()),
        hunting: ActiveMonsterId::default(),
        name: k.name.clone(),
        wornitem: ItemNo::default(),
        weapon: k.weapon,
        items: std::array::from_fn(|_| (ItemNo::default(), 0)),
        cash: Coins::default(),
        maxhp: k.maxhp,
        alignment: k.alignment,
        followpct: k.followpct,
        dr: k.dr,
        ac: k.ac,
        exp: k.exp,
        maxenergy: k.energy as i16,
        friendly: 0,
        genroom: RoomId(ROOM),
        inactive: 0,
        flags: ActiveMonsterFlags::empty(),
        montype: k.montype,
        target: -1,
        hpregen: 0,
        lastdir: 0,
        lastupd: 0,
        grouprole: GroupRole::Solo,
        spells: std::array::from_fn(|_| ActiveSpell {
            spell: Default::default(),
            value: 0,
            rounds: 0,
        }),
        downrounds: 0,
        created: 0,
    });
    let rs = w.room_state.get_mut(&RoomKey::new(1, ROOM)).unwrap();
    *rs.monsters.iter_mut().find(|m| m.0 == 0).unwrap() = number;
    number
}

pub(crate) fn lines(w: &mut World, id: SessionId) -> Vec<String> {
    let prompt = silvermere_engine::tell::prompt_bytes(w, id);
    let palette = silvermere_engine::tell::palette_of(w, id);
    let s = w.session_mut(id).unwrap();
    let b = s.output.render(palette, &prompt);
    s.output.clear();
    let text = String::from_utf8_lossy(&b).replace("\x1b[79D\x1b[K", "\r\n");
    String::from_utf8_lossy(&strip_ansi(text.as_bytes()))
        .split("\r\n")
        .map(|l| l.trim_matches([' ', '\u{7}']).to_owned())
        .filter(|l| !l.is_empty() && !l.starts_with("[HP="))
        .collect()
}

/// One blow's energy for the caster, as the round's first blow.
fn blow_energy(w: &mut World, id: SessionId) -> u32 {
    let mut c = plr(w, id).clone();
    c.settings.insert(PlayerSettings::CANCAST);
    let (f, _) = player_side(w, id, &c, Move::Attack, Move::Attack, AlignGroup::Neutral).unwrap();
    f.energy
}

/// Sets up a fight with `spell` and runs one auto-combat round for the caster.
fn one_round(w: &mut World, id: SessionId, spell: SpellNo) -> (Vec<String>, ActiveMonsterId) {
    let m = spawn(w);
    engage(
        w,
        id,
        Some(AutoTarget::Monster(m)),
        Move::Attack,
        Some(spell),
    );
    lines(w, id);
    do_autocombat_for_user(w, id);
    (lines(w, id), m)
}

#[test]
fn only_single_target_attack_spells_are_spellstrikes() {
    assert!(is_spellstrike_spell(&spell(
        1,
        "magic missile",
        SpellTarget::MonsterOrSingle
    )));
    assert!(!is_spellstrike_spell(&spell(
        2,
        "fireball",
        SpellTarget::FullAttack
    )));
    let mut free = spell(3, "free bolt", SpellTarget::Single);
    free.energy = 0;
    assert!(!is_spellstrike_spell(&free));
}

#[test]
fn a_successful_spellstrike_casts_then_strikes() {
    let mut w = world();
    let id = caster(&mut w, WAR_MAGE, true);
    let blow = blow_energy(&mut w, id);
    let (out, m) = one_round(&mut w, id, MMIS);
    assert_eq!(
        out.first().map(String::as_str),
        Some("You channel magic missile through your longsword!"),
        "{out:?}"
    );
    assert!(
        out.iter().any(|l| l.starts_with("You cast magic missile")),
        "{out:?}"
    );
    // the cast cost 90% of a blow; whole blows took the rest of the pool
    let p = plr(&mut w, id).clone();
    let after_cast = 1000 - blow * 90 / 100;
    let spent_on_blows = after_cast - u32::from(p.energy);
    assert_eq!(spent_on_blows % blow, 0, "blows of {blow}: {out:?}");
    assert!(u32::from(p.energy) < blow, "the round's energy is used up");
    assert!(
        spent_on_blows / blow >= 1,
        "at least one blow after the cast"
    );
    assert_eq!(p.mana, 36, "a success charges the full mana");
    assert!(
        w.active_monsters
            .values()
            .any(|x| x.number == m && x.hp < 9999)
    );
    // the fight goes on as spellstrikes, with one place in the ring
    let e = w.autocombat.entry(id);
    assert!(e.casting);
    assert_eq!(e.spell, MMIS);
    assert_eq!(w.autocombat.queue().iter().filter(|&&q| q == id).count(), 1);
}

#[test]
fn a_failed_spellstrike_costs_less_and_still_strikes() {
    let mut w = world();
    let id = caster(&mut w, WAR_MAGE, true);
    plr(&mut w, id).spellcast = -500;
    let blow = blow_energy(&mut w, id);
    let (out, _) = one_round(&mut w, id, MMIS);
    assert_eq!(out[0], "You channel magic missile through your longsword!");
    assert!(
        out.iter()
            .any(|l| l == "You attempt to cast magic missile, but fail."),
        "{out:?}"
    );
    let p = plr(&mut w, id).clone();
    let after_cast = 1000 - blow * 30 / 100;
    assert_eq!((after_cast - u32::from(p.energy)) % blow, 0, "{out:?}");
    assert_eq!(p.mana, 38, "a failure charges half the mana");
}

#[test]
fn out_of_mana_the_fight_turns_to_melee() {
    let mut w = world();
    let id = caster(&mut w, WAR_MAGE, true);
    plr(&mut w, id).mana = 1;
    let (out, _) = one_round(&mut w, id, MMIS);
    assert_eq!(
        out[0], "You do not have enough mana to cast that spell.",
        "{out:?}"
    );
    assert!(!out.iter().any(|l| l.contains("channel")));
    assert!(
        plr(&mut w, id).energy < 1000,
        "the round is fought, not lost"
    );
    let e = w.autocombat.entry(id);
    assert!(!e.casting, "melee from now on");
}

#[test]
fn unarmed_area_and_other_classes_cast_as_stock() {
    // no weapon
    let mut w = world();
    let id = caster(&mut w, WAR_MAGE, false);
    let (out, _) = one_round(&mut w, id, MMIS);
    assert!(!out.iter().any(|l| l.contains("channel")), "{out:?}");
    // a class without Spellstrike
    let mut w = world();
    let id = caster(&mut w, PLAIN_MAGE, true);
    let (out, _) = one_round(&mut w, id, MMIS);
    assert!(!out.iter().any(|l| l.contains("channel")), "{out:?}");
    assert_eq!(
        plr(&mut w, id).energy,
        0,
        "a stock casting round spends it all"
    );
}
