//! Fork (design/warden.md): the Newhaven stable on the imported stock world, through the
//! real server. The content (the Warden class, the stable's rooms, NPCs, shop, leashes and
//! script) is installed when the server starts. A Warden buys a starter leash, is refused a
//! second, bonds the companion by using the leash in the stables, sees and names it.

use crate::common;

use common::m3::{Player, edit_char, place, start_manual, stored, team_db};
use silvermere_db::Db;
use silvermere_types::fork;
use silvermere_types::ids::ClassNo;

fn ability(c: &silvermere_types::records::Character, a: silvermere_types::enums::Ability) -> i16 {
    c.abils.iter().find(|s| s.0 == a).map_or(0, |s| s.1)
}

#[tokio::test]
async fn a_warden_bonds_a_companion_at_the_newhaven_stable() {
    let (_dir, url) = team_db();
    edit_char(&url, "Aldric", |c| c.class = ClassNo(17)).await;
    // from the Village Entrance, through the yard into the stables: the NPCs appear as a
    // player walks in
    place(&url, "Aldric", 2140).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    a.walk_ticking(&r, "e", "Stable Yard").await;
    a.walk_ticking(&r, "e", "Warden Stables").await;

    let out = a.cmd("ask deel leashes").await;
    assert!(out.contains("Birds are fast"), "{out}");
    let out = a.cmd("list").await;
    for leash in [
        "hound's leash",
        "hawkling's leash",
        "rattlesnake's leash",
        "drop bear's leash",
    ] {
        assert!(out.contains(leash), "{leash}: {out}");
    }
    let out = a.cmd("buy hound's leash").await;
    assert!(out.contains("You just bought"), "{out}");
    let out = a.cmd("buy hawkling's leash").await;
    assert!(out.contains("One starter beast to a Warden"), "{out}");
    let out = a.cmd("use hound's leash").await;
    assert!(out.contains("The hound is yours"), "{out}");
    let out = a.cmd("pet").await;
    assert!(
        out.contains("Your hunting hound fights at your side"),
        "{out}"
    );
    let out = a.cmd("pet name Fang").await;
    assert!(
        out.contains("Your hunting hound is now called Fang."),
        "{out}"
    );
    // one starter companion: a second bond is refused
    let out = a.cmd("use hound's leash").await;
    assert!(out.contains("bonded your starter beast already"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();

    let c = stored(&url, "Aldric").await;
    assert_eq!(
        ability(&c, fork::PET_KIND),
        1,
        "bonded to the hunting hound"
    );
    assert_eq!(ability(&c, fork::STARTER_BONDED), 1);
    assert_eq!(ability(&c, fork::STARTER_LEASH), 1);
    let db = Db::connect_and_migrate(&url).await.unwrap();
    let account = db
        .characters()
        .by_first_name("Aldric")
        .await
        .unwrap()
        .unwrap()
        .account;
    let pets = db.pets().of_account(account).await.unwrap();
    assert_eq!(pets.len(), 1);
    assert_eq!((pets[0].name.as_str(), pets[0].kind), ("Fang", 1));
    db.close().await;
}

#[tokio::test]
async fn the_stable_yard_and_old_bannon() {
    let (_dir, url) = team_db();
    place(&url, "Aldric", 2140).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    a.walk_ticking(&r, "e", "Stable Yard").await;
    let out = a.cmd("look").await;
    assert!(out.contains("Newhaven, Stable Yard"), "{out}");
    assert!(out.contains("Old Bannon"), "{out}");
    let out = a.cmd("ask bannon pets").await;
    assert!(out.contains("talk to Deel Barmny"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let db = Db::connect_and_migrate(&url).await.unwrap();
    let yard = db
        .rooms()
        .get(silvermere_types::ids::RoomKey::new(1, 2140))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(yard.exits[2].target.0, 9000, "east of the Village Entrance");
    db.close().await;
}

/// Fork (design/stealth-casters.md): the Newhaven weapon shop is the fork's copy (901), which
/// sells a secondhand dagger beside the club and quarterstaff, so a new Assassin (sharp
/// weapons only) has something to wield.
#[tokio::test]
async fn a_new_assassin_buys_and_wields_the_secondhand_dagger() {
    let (_dir, url) = team_db();
    edit_char(&url, "Aldric", |c| c.class = ClassNo(19)).await;
    place(&url, "Aldric", 2141).await;
    let r = start_manual(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    let out = a.cmd("list").await;
    for item in ["club", "quarterstaff", "secondhand dagger"] {
        assert!(out.contains(item), "{item}: {out}");
    }
    let out = a.cmd("buy secondhand dagger").await;
    assert!(out.contains("You just bought"), "{out}");
    let out = a.cmd("wield secondhand dagger").await;
    assert!(!out.contains("can't use"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
    let c = stored(&url, "Aldric").await;
    assert_eq!(c.weapon, silvermere_types::ids::ItemNo(9004), "wielded");
    let db = Db::connect_and_migrate(&url).await.unwrap();
    let dagger = db
        .items()
        .get(silvermere_types::ids::ItemNo(9004))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (dagger.wpntype, dagger.mindmg, dagger.maxdmg, dagger.speed),
        (silvermere_types::enums::WeaponType::OneHandSharp, 3, 7, 900)
    );
    let room = db
        .rooms()
        .get(silvermere_types::ids::RoomKey::new(1, 2141))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(room.shopnum.0, 901);
    db.close().await;
}

/// Fork: the universal trainers a new character meets, Newhaven's Adventurer's Guild (1/2147)
/// and the Adventurer's Guild Universal Trainer (1/1376), train the fork's classes too: a
/// War Mage and a Warden each train level 2 at both.
#[tokio::test]
async fn the_universal_trainers_train_war_mages_and_wardens() {
    for (class, title) in [(16, "Apprentice Soldat"), (17, "Scout")] {
        for room in [2147, 1376] {
            let (_dir, url) = team_db();
            edit_char(&url, "Aldric", |c| {
                c.class = ClassNo(class);
                c.level = 1;
                c.expbil = 0;
                c.exprem = 1_000_000;
                c.coins.gold = 1_000;
            })
            .await;
            place(&url, "Aldric", room).await;
            let r = start_manual(&url).await;
            let mut a = Player::enter(&r, "Aldric").await;
            let out = a.cmd("train").await;
            assert!(
                out.contains("receive training to attain level 2"),
                "class {class} at 1/{room}: {out}"
            );
            a.leave_ticking(&r).await;
            r.stop().await.unwrap();
            let c = stored(&url, "Aldric").await;
            assert_eq!(c.level, 2, "class {class} at 1/{room}");
            assert_eq!(c.title, title, "class {class}: its own level 2 title");
        }
    }
}

/// Fork: a world where an earlier install copied the NPCs with their template's alive count
/// (Nathaniel's 1, of 1 allowed) still gets them: the next start sets the count to what is
/// really in the world, and they appear as a player walks in.
#[tokio::test]
async fn the_stable_npcs_appear_on_a_world_with_a_stale_alive_count() {
    let (_dir, url) = team_db();
    place(&url, "Aldric", 2140).await;
    let db = Db::connect_and_migrate(&url).await.unwrap();
    for n in [9000, 9001] {
        db.monster_state()
            .put(&silvermere_db::repo::MonsterState {
                number: silvermere_types::ids::MonsterNo(n),
                curalive: 1,
                killed_at: 0,
            })
            .await
            .unwrap();
    }
    db.close().await;
    let r = start_reading_the_world(&url).await;
    let mut a = Player::enter(&r, "Aldric").await;
    let out = a.walk_ticking(&r, "e", "Stable Yard").await;
    assert!(out.contains("Old Bannon"), "{out}");
    let out = a.walk_ticking(&r, "e", "Warden Stables").await;
    assert!(out.contains("Deel Barmny"), "{out}");
    a.leave_ticking(&r).await;
    r.stop().await.unwrap();
}

/// Starts the server as a real start does: the content installed, then the world read from
/// the database. (The shared helpers hand the server a world read once per template, before
/// any start's install, which can't show what an install corrects.) Ticks come from the test.
async fn start_reading_the_world(url: &str) -> silvermere::Running {
    let mut cfg = silvermere::config::Config::default();
    cfg.server.telnet_bind = "127.0.0.1:0".into();
    cfg.server.admin_bind = "127.0.0.1:0".into();
    cfg.database.url = url.to_owned();
    cfg.data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    cfg.game.sec_fast = 86_400;
    silvermere_text::package::load_and_install(&silvermere_testkit::stock::assets_dir())
        .expect("the stock package's text");
    silvermere::serve_snapshot_in_process(
        cfg,
        None,
        silvermere_engine::SystemClock,
        silvermere_engine::SeededRng::new(7),
    )
    .await
    .expect("starts")
}
