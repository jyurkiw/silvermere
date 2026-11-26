//! Fork (design/class-trainers.md): the fork classes' trainers, the Silvermere stables, the
//! Temple's hidden room, and the tomb wings off the Crypt's Shadowed Hall, on the imported
//! stock world through the real server. The content is installed when the server starts.

use crate::common;

use common::m3::{Player, edit_char, place, start_manual, stored, team_db};
use silvermere_types::fork;
use silvermere_types::ids::{ClassNo, ItemNo};

/// ASK, and the part of the answer a script adds, which comes after the lead-in's prompt.
async fn ask(a: &mut Player, line: &str, then: &str) -> String {
    let at = a.c.position();
    a.cmd(line).await;
    let got = a.c.plain_from(at);
    if got.contains(then) {
        return got;
    }
    a.wait_text(at, then).await
}

/// A move that may be refused: the line, a few ticks, and everything that came back.
async fn try_move(a: &mut Player, r: &silvermere::Running, dir: &str) -> String {
    let at = a.c.position();
    a.say(dir).await;
    for _ in 0..3 {
        common::m3::tick(r, 1).await;
        let _ = a.drain().await;
    }
    a.c.plain_from(at)
}

fn ability(c: &silvermere_types::records::Character, a: silvermere_types::enums::Ability) -> i16 {
    c.abils.iter().find(|s| s.0 == a).map_or(0, |s| s.1)
}

/// A character of `class` at `level`, with experience and gold to spare, at `room`.
async fn ready(url: &str, class: u16, level: u16, room: u32) {
    edit_char(url, "Aldric", |c| {
        c.class = ClassNo(class);
        c.level = level;
        c.expbil = 0;
        c.exprem = 2_000_000_000;
        c.coins.gold = 5_000;
        // a lit lantern: the Crypt's rooms are dark (light -175), as stock's
        c.items[9] = ItemNo(176);
        c.itemuses[9] = 2400;
        c.readied = 9;
        c.light = 175;
    })
    .await;
    place(url, "Aldric", room).await;
}

#[tokio::test]
async fn each_fork_class_trains_at_its_own_trainer() {
    // class, start room, the way in, the level-2 title
    let routes: [(u16, u32, &[(&str, &str)], &str); 4] = [
        (16, 398, &[("u", "War Mage Training Room")], "Apprentice Soldat"),
        (19, 396, &[("n", "Ink Shop")], "Footpad"),
        (
            20,
            491,
            &[
                ("w", "Adventurer's Guild, Halls of Training"),
                ("n", "Dervish Training Room"),
            ],
            "Dancer",
        ),
        (
            17,
            165,
            &[
                ("e", "Silvermere Stables, Yard"),
                ("e", "Silvermere Stables"),
                ("e", "Warden Training Room"),
            ],
            "Scout",
        ),
    ];
    for (class, start, way, title) in routes {
        let (_dir, url) = team_db();
        ready(&url, class, 1, start).await;
        let r = start_manual(&url).await;
        let mut a = Player::enter(&r, "Aldric").await;
        for (dir, room) in way {
            a.walk_ticking(&r, dir, room).await;
        }
        let out = a.cmd("train").await;
        assert!(
            out.contains("receive training to attain level 2"),
            "class {class}: {out}"
        );
        a.leave_ticking(&r).await;
        r.stop().await.unwrap();
        let c = stored(&url, "Aldric").await;
        assert_eq!((c.level, c.title.as_str()), (2, title), "class {class}");
    }
}

#[tokio::test]
async fn another_class_cannot_train_at_a_fork_trainer() {
    let (_dir, url) = team_db();
    ready(&url, 1, 1, 398).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    a.walk_ticking(&r, "u", "War Mage Training Room").await;
    let out = a.cmd("train").await;
    assert!(!out.contains("attain level 2"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    assert_eq!(stored(&url, "Aldric").await.level, 1);
}

#[tokio::test]
async fn the_temple_catch_and_lever_open_the_avenger_room_in_order() {
    let (_dir, url) = team_db();
    ready(&url, 18, 1, 2316).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    // the lever alone does nothing
    a.cmd("pull lever").await;
    let out = a.cmd("n").await;
    assert!(!out.contains("Avenger Training Room"), "{out}");
    let out = a.cmd("push catch").await;
    assert!(out.contains("catch give under your fingers"), "{out}");
    let out = a.cmd("pull lever").await;
    assert!(out.contains("swings inward"), "{out}");
    a.walk_ticking(&r, "n", "Avenger Training Room").await;
    let out = a.cmd("train").await;
    assert!(out.contains("receive training to attain level 2"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
}

#[tokio::test]
async fn the_preacher_speaks_of_blood_only_to_an_avenger() {
    for (class, avenger) in [(18, true), (1, false)] {
        let (_dir, url) = team_db();
        // the preacher appears as a player walks into the foyer
        ready(&url, class, 1, 324).await;
        let r = start_manual(&url).await;
        let mut a = Player::enter(&r, "Aldric").await;
        a.walk_ticking(&r, "w", "Temple Foyer").await;
        if avenger {
            let out = ask(&mut a, "ask preacher forgiveness", "or in blood").await;
            assert!(out.contains("paid for in gold"), "{out}");
            ask(&mut a, "ask preacher blood", "PULL the LEVER").await;
        } else {
            let out = a.cmd("ask preacher forgiveness").await + &a.drain().await;
            assert!(out.contains("paid for in gold"), "{out}");
            assert!(!out.contains("or in blood"), "{out}");
            let out = ask(&mut a, "ask preacher blood", "I said nothing about blood").await;
            assert!(!out.contains("LEVER"), "{out}");
        }
        a.leave_ticking(&r).await;
        r.stop().await.unwrap();
    }
}

#[tokio::test]
async fn the_dark_robed_figure_trains_only_assassins() {
    for (class, assassin) in [(19, true), (12, false)] {
        let (_dir, url) = team_db();
        ready(&url, class, 1, 396).await;
        let r = start_manual(&url).await;
        let mut a = Player::enter(&r, "Aldric").await;
        a.walk_ticking(&r, "n", "Ink Shop").await;
        let answer = if assassin {
            "I can see to your training"
        } else {
            "I make inks for Aiken"
        };
        let out = ask(&mut a, "ask figure ink", answer).await;
        let other = if assassin { "inks for Aiken" } else { "see to your training" };
        assert!(!out.contains(other), "{out}");
        a.leave_ticking(&r).await;
        r.stop().await.unwrap();
    }
}

#[tokio::test]
async fn the_silvermere_stables_sell_leashes_and_a_new_one_replaces_the_companion() {
    let (_dir, url) = team_db();
    ready(&url, 17, 5, 165).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    a.walk_ticking(&r, "e", "Silvermere Stables, Yard").await;
    let out = a.cmd("ask corra leashes").await;
    assert!(out.contains("shadow mastiff"), "{out}");
    a.walk_ticking(&r, "e", "Silvermere Stables").await;
    let out = a.cmd("list").await;
    for leash in ["hound's leash", "war hound's leash", "shadow mastiff's leash"] {
        assert!(out.contains(leash), "{leash}: {out}");
    }
    let out = a.cmd("buy war hound's leash").await;
    assert!(out.contains("You just bought"), "{out}");
    let out = a.cmd("use war hound's leash").await;
    assert!(out.contains("broad-chested war hound"), "{out}");
    let out = a.cmd("pet name Rex").await;
    assert!(out.contains("now called Rex"), "{out}");
    // a weaker beast replaces the stronger one, unnamed
    a.cmd("buy hound's leash").await;
    let out = a.cmd("use hound's leash").await;
    assert!(out.contains("hunting hound"), "{out}");
    let out = a.cmd("pet").await;
    assert!(out.contains("Your hunting hound fights"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let c = stored(&url, "Aldric").await;
    assert_eq!(ability(&c, fork::PET_KIND), 1, "the hunting hound now");
}

#[tokio::test]
async fn only_its_classes_enter_a_tomb_wing() {
    for (class, enters) in [(18, true), (1, false)] {
        let (_dir, url) = team_db();
        ready(&url, class, 10, 1421).await;
        let r = start_manual(&url).await;
        let mut a = Player::enter(&r, "Aldric").await;
        let out = try_move(&mut a, &r, "nw").await;
        assert_eq!(out.contains("Marble Passage"), enters, "class {class}: {out}");
        assert_eq!(out.contains("may not go through"), !enters, "class {class}: {out}");
        a.leave_ticking(&r).await;
        r.stop().await.unwrap();
    }
}

#[tokio::test]
async fn a_tomb_spirit_gives_the_class_weapon_once_and_the_super_trainer_trains_level_11() {
    let (_dir, url) = team_db();
    // the stealth casters' hub, then the Avenger's tomb, where its spirit appears
    ready(&url, 18, 10, 9124).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    a.walk_ticking(&r, "nw", "Large Tomb").await;
    ask(&mut a, "ask spirit weapon", "Take this, and carry it well").await;
    ask(&mut a, "ask spirit weapon", "You have had my gift already").await;
    let out = a.cmd("train").await;
    assert!(out.contains("receive training to attain level 11"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let c = stored(&url, "Aldric").await;
    assert!(c.items.contains(&ItemNo(9033)), "the censer of judgement");
    assert_eq!(c.level, 11);
    assert_eq!(ability(&c, fork::CLASS_WEAPON_GIVEN), 1);
}

#[tokio::test]
async fn the_hub_lets_each_class_into_its_own_tomb_only() {
    let (_dir, url) = team_db();
    ready(&url, 19, 10, 9124).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    // the Avenger's tomb is barred to an Assassin; its own is north
    let out = try_move(&mut a, &r, "nw").await;
    assert!(out.contains("may not go through"), "{out}");
    a.walk_ticking(&r, "n", "Large Tomb").await;
    ask(&mut a, "ask spirit weapon", "Take this, and carry it well").await;
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let c = stored(&url, "Aldric").await;
    assert!(c.items.contains(&ItemNo(9034)), "the hexknife");
    assert!(!c.items.contains(&ItemNo(9033)));
}

#[tokio::test]
async fn the_kris_is_held_only_in_the_off_hand() {
    let (_dir, url) = team_db();
    ready(&url, 20, 10, 2141).await;
    edit_char(&url, "Aldric", |c| {
        c.items[0] = ItemNo(9031);
        c.items[1] = ItemNo(90);
    })
    .await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    let out = a.cmd("wield kris").await;
    assert!(out.contains("balanced for the off-hand"), "{out}");
    a.cmd("wield club").await;
    let out = a.cmd("offhand kris").await;
    assert!(out.contains("in your off-hand"), "{out}");
    // the main hand empties: the kris stays where it is
    let out = a.cmd("remove club").await;
    assert!(!out.contains("shift"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let c = stored(&url, "Aldric").await;
    assert_eq!(c.weapon, ItemNo(0));
    assert!(c.worn.contains(&ItemNo(9031)));
}
