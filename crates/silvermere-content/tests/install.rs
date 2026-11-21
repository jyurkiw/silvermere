//! Installing into a database: idempotent, refuses numbers other content holds, and
//! applies changes to classes already there.

use silvermere_content::{Content, InstallError, install};
use silvermere_db::TestDb;
use silvermere_types::enums::{Ability, ArmourType, Magery, WeaponClass};
use silvermere_types::fork;
use silvermere_types::ids::{ClassNo, TextBlockNo};
use silvermere_types::records::{Class, TextBlock};
use silvermere_types::sizes::NRECABILS;

const FIXTURE: &str = r#"
[[class]]
number = 16
name = "Tester"
hp = "5-9"
exp = 220
magic = "Mage-1"
fighting = 4
weapons = "All"
armour = "Chainmail"
abilities = [{ name = "Spellstrike", value = 30 }]
[class.titles]
block = 3019
ranks = [{ from = 1, title = "Recruit" }, { from = 10, title = "Veteran" }]

[[class_change]]
name = "Warlock"
add_abilities = [{ name = "Spellstrike", value = 30 }]
"#;

fn warlock() -> Class {
    let mut abilities = [(Ability::None, 0); NRECABILS];
    abilities[0] = (Ability::Bash, 0);
    Class {
        number: ClassNo(11),
        name: "Warlock".into(),
        basehp: 4,
        hplevel: 4,
        exp: 85,
        attmult: 0,
        maxenergy: 0,
        restored: 0,
        abilities,
        magery: Magery::Mage,
        magelvl: 2,
        weapon: WeaponClass::AnyOneHand,
        armour: ArmourType::Chainmail,
        combat: 5,
        titles: TextBlockNo(3011),
    }
}

fn fixture() -> Content {
    Content::from_files(&[("classes.toml", FIXTURE)]).unwrap()
}

async fn world() -> TestDb {
    let db = TestDb::sqlite_temp().await.unwrap();
    db.classes().put(&warlock()).await.unwrap();
    db
}

#[tokio::test]
async fn installs_new_classes_titles_and_changes() {
    let db = world().await;
    let r = install(&db, &fixture()).await.unwrap();
    assert_eq!((r.classes, r.titles, r.changes), (1, 1, 1));
    let t = db.classes().get(ClassNo(16)).await.unwrap().unwrap();
    assert_eq!(t.name, "Tester");
    assert!(t.abilities.contains(&(fork::SPELLSTRIKE, 30)));
    let block = db
        .text_blocks()
        .part(0, TextBlockNo(3019))
        .await
        .unwrap()
        .unwrap();
    assert!(block.text.starts_with("Recruit\n"));
    let w = db.classes().get(ClassNo(11)).await.unwrap().unwrap();
    assert!(w.abilities.contains(&(fork::SPELLSTRIKE, 30)));
    assert!(
        w.abilities.contains(&(Ability::Bash, 0)),
        "the rest of the class is kept"
    );
}

#[tokio::test]
async fn installing_twice_gives_the_same_rows() {
    let db = world().await;
    install(&db, &fixture()).await.unwrap();
    let once = (
        db.classes().all().await.unwrap(),
        db.text_blocks().all().await.unwrap(),
    );
    install(&db, &fixture()).await.unwrap();
    let twice = (
        db.classes().all().await.unwrap(),
        db.text_blocks().all().await.unwrap(),
    );
    assert_eq!(once, twice);
    let w = db.classes().get(ClassNo(11)).await.unwrap().unwrap();
    let n = w
        .abilities
        .iter()
        .filter(|s| s.0 == fork::SPELLSTRIKE)
        .count();
    assert_eq!(n, 1, "a change sets an ability once, however often it runs");
}

#[tokio::test]
async fn a_reimport_is_repaired_by_the_next_install() {
    let db = world().await;
    install(&db, &fixture()).await.unwrap();
    // what `import --content-only` does to a stock class: it is written back as stock
    db.classes().put(&warlock()).await.unwrap();
    install(&db, &fixture()).await.unwrap();
    let w = db.classes().get(ClassNo(11)).await.unwrap().unwrap();
    assert!(w.abilities.contains(&(fork::SPELLSTRIKE, 30)));
}

#[tokio::test]
async fn a_number_held_by_other_content_is_refused() {
    let db = world().await;
    let mut other = warlock();
    other.number = ClassNo(16);
    other.name = "Samurai".into();
    db.classes().put(&other).await.unwrap();
    let e = install(&db, &fixture()).await.unwrap_err();
    assert!(matches!(e, InstallError::Taken(_)), "{e}");
    let kept = db.classes().get(ClassNo(16)).await.unwrap().unwrap();
    assert_eq!(kept.name, "Samurai", "nothing is overwritten");
}

#[tokio::test]
async fn a_titles_block_already_in_use_is_refused() {
    let db = world().await;
    let theirs = TextBlock {
        part: 0,
        number: TextBlockNo(3019),
        link: TextBlockNo(0),
        text: "someone else's\n".into(),
    };
    db.text_blocks().put(&theirs).await.unwrap();
    let r = install(&db, &fixture()).await;
    assert!(matches!(r, Err(InstallError::Taken(_))), "{r:?}");
}

#[tokio::test]
async fn a_change_to_a_missing_class_is_refused() {
    let db = TestDb::sqlite_temp().await.unwrap();
    let r = install(&db, &fixture()).await;
    assert!(matches!(r, Err(InstallError::Change(_))), "{r:?}");
}
