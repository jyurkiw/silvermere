//! The content built into the binary: it parses, and the War Mage comes out as designed
//! (design/war-mage.md).

use silvermere_content::{Content, TITLE_LEVELS, help};
use silvermere_types::enums::{Ability, ArmourType, Magery, WeaponClass};
use silvermere_types::fork;
use silvermere_types::ids::{ClassNo, TextBlockNo};
use silvermere_types::records::Class;

fn war_mage(c: &Content) -> &Class {
    c.classes
        .iter()
        .find(|c| c.name == "War Mage")
        .expect("the War Mage is in content/classes.toml")
}

#[test]
fn the_embedded_content_is_valid() {
    Content::embedded().expect("content/ parses and checks");
}

#[test]
fn the_war_mage_record_matches_the_design() {
    let c = Content::embedded().unwrap();
    let w = war_mage(&c);
    assert_eq!(w.number, ClassNo(16));
    assert_eq!((w.basehp, w.hplevel), (5, 4), "5-9 HP per level");
    assert_eq!(w.exp, 120, "220% experience");
    assert_eq!((w.magery, w.magelvl), (Magery::Mage, 1));
    assert_eq!(w.combat, 6, "fighting 4, a Warrior's");
    assert_eq!(w.weapon, WeaponClass::All);
    assert_eq!(w.armour, ArmourType::Chainmail);
    assert_eq!(w.titles, TextBlockNo(3019));
    let has = |a, v| w.abilities.contains(&(a, v));
    assert!(has(Ability::Bash, 0));
    assert!(has(fork::SPELLSTRIKE, 30));
    assert!(has(fork::SPELLSTRIKE_SUCCESS_OFF, 10));
    assert!(has(fork::SPELLSTRIKE_FAIL_OFF, 40));
    assert!(has(fork::NO_OFF_HAND, 0));
}

#[test]
fn the_war_mage_titles_cover_every_level() {
    let c = Content::embedded().unwrap();
    let block = c
        .titles
        .iter()
        .find(|t| t.number == TextBlockNo(3019))
        .unwrap();
    let lines: Vec<&str> = block.text.lines().collect();
    assert_eq!(lines.len(), usize::from(TITLE_LEVELS));
    assert_eq!(lines[0], "Apprentice Recruit");
    assert_eq!(lines[1], "Apprentice Soldat");
    assert_eq!(lines[4], "Arcane Brigadier");
    assert_eq!(lines[24], "Chief Sergeant");
    assert_eq!(lines[69], "Arch-Colonel");
    assert_eq!(lines[109], "Arch-Colonel");
}

#[test]
fn the_war_mage_help_topic_draws_its_stats_from_the_record() {
    let c = Content::embedded().unwrap();
    let topics = help::topics(&c).unwrap();
    let t = topics.iter().find(|t| t.words == "War Mages").unwrap();
    assert!(t.text.contains("WAR MAGES"));
    assert!(t.text.contains("EXP: 220%"));
    assert!(
        t.text
            .contains("HP/Level: 5-9  Magic: Mage-1  Combat: Good  Weapons: Any  Armour: Chain")
    );
    assert!(t.text.contains("Abilities: Spellstrike."));
    // every line of the box is the same width
    let widths: std::collections::BTreeSet<usize> = t
        .text
        .split('\r')
        .filter(|l| !l.is_empty())
        .map(|l| l.chars().count())
        .collect();
    assert_eq!(widths.len(), 1, "{widths:?}");
    assert!(topics.iter().any(|t| t.words == "Spellstrike"));
}

#[test]
fn bad_content_is_refused() {
    let bad = |toml: &str| Content::from_files(&[("classes.toml", toml)]).is_err();
    let base = "[[class]]\nnumber = 20\nname = \"Tester\"\nhp = \"5-9\"\nexp = 150\n\
                magic = \"Mage-1\"\nfighting = 3\nweapons = \"All\"\narmour = \"Chainmail\"\n";
    assert!(Content::from_files(&[("classes.toml", base)]).is_ok());
    assert!(bad(&base.replace("5-9", "9-5")));
    assert!(bad(&base.replace("Mage-1", "Mage-4")));
    assert!(bad(&base.replace("fighting = 3", "fighting = 6")));
    assert!(bad(&base.replace("\"All\"", "\"Swords\"")));
    assert!(bad(&format!(
        "{base}abilities = [{{ name = \"Nonesuch\" }}]\n"
    )));
    assert!(
        bad(&format!("{base}colour = \"red\"\n")),
        "unknown keys are refused"
    );
    assert!(Content::from_files(&[("rooms.toml", "")]).is_err());
}

#[test]
fn the_warden_record_matches_the_design() {
    let c = Content::embedded().unwrap();
    let w = c.classes.iter().find(|c| c.name == "Warden").unwrap();
    assert_eq!(w.number, ClassNo(17));
    assert_eq!((w.basehp, w.hplevel), (7, 4), "7-11 HP per level");
    assert_eq!(w.exp, 40, "140% experience");
    assert_eq!((w.magery, w.magelvl), (Magery::None, 0));
    assert_eq!(w.combat, 5, "fighting 3");
    assert_eq!(w.weapon, WeaponClass::AnyTwoHand);
    assert_eq!(w.armour, ArmourType::StuddedRigidLeather);
    assert_eq!(w.titles, TextBlockNo(3020));
    assert!(w.abilities.contains(&(fork::CHAINMAIL_AT_LEVEL, 10)));
    assert!(w.abilities.contains(&(fork::SCALEMAIL_AT_LEVEL, 20)));
    assert!(w.abilities.contains(&(fork::RANGED_WEAPONS, 0)));
    let block = c
        .titles
        .iter()
        .find(|t| t.number == TextBlockNo(3020))
        .unwrap();
    let lines: Vec<&str> = block.text.lines().collect();
    assert_eq!(
        (lines[0], lines[1], lines[69]),
        ("Cub", "Scout", "Huntsman")
    );
    let topics = help::topics(&c).unwrap();
    let t = topics.iter().find(|t| t.words == "Wardens").unwrap();
    assert!(
        t.text.contains(
            "HP/Level: 7-11  Magic: None  Combat: Average  Weapons: 2-H  Armour: Leather"
        )
    );
}
