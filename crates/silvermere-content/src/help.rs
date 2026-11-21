//! The content's HELP topics, ready to add to the game's text when it loads.
//!
//! A class topic is drawn in the box the stock class topics use, from the class's own
//! record, so its stat line can't drift from the numbers.

use silvermere_text::catalogue::HelpTopic;
use silvermere_text::package::to_template;
use silvermere_types::enums::{ArmourType, Magery, WeaponClass};
use silvermere_types::records::Class;

use crate::Content;

/// The inside width of the box, between the borders.
const INNER: usize = 75;

/// The content's topics, in file order: class topics first, then the others. Their keys
/// are `FORKnT` / `FORKnB`, apart from the stock `HL1` keys.
pub fn topics(content: &Content) -> Result<Vec<HelpTopic>, String> {
    let mut out = Vec::new();
    let mut n = 0;
    let mut push = |words: String, raw: String| -> Result<(), String> {
        n += 1;
        out.push(HelpTopic {
            key: format!("FORK{n}T"),
            body_key: format!("FORK{n}B"),
            words,
            text: to_template(&raw, &[]).map_err(|e| format!("topic {n}: {e}"))?,
        });
        Ok(())
    };
    for t in &content.help.class_topic {
        let class = content
            .classes
            .iter()
            .find(|c| c.name == t.class)
            .ok_or_else(|| format!("{}: no such class in the content", t.class))?;
        let words = t
            .words
            .clone()
            .unwrap_or_else(|| format!("{}s", class.name));
        push(
            words.clone(),
            class_box(class, &words, &t.text, &t.abilities),
        )?;
    }
    for t in &content.help.topic {
        push(t.words.clone(), t.text.clone())?;
    }
    Ok(out)
}

fn rule(left: char, right: char) -> String {
    format!("{left}{}{right}\n", "═".repeat(INNER))
}

fn row(text: &str) -> String {
    let len = text.chars().count();
    format!("║ {text}{} ║\n", " ".repeat(INNER.saturating_sub(len + 2)))
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for para in text.split("\n\n") {
        let mut line = String::new();
        for word in para.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

/// The help's word for a class's fighting (`combat` 3-7).
fn fighting_word(combat: i16) -> &'static str {
    match combat {
        ..=3 => "Poor",
        4 => "Fair",
        5 => "Average",
        6 => "Good",
        _ => "Excellent",
    }
}

fn weapons_word(w: WeaponClass) -> &'static str {
    match w {
        WeaponClass::All => "Any",
        WeaponClass::AnyOneHand => "1-H",
        WeaponClass::AnyTwoHand => "2-H",
        WeaponClass::AnyBlunt => "Blunt",
        WeaponClass::AnySharp => "Sharp",
        WeaponClass::OneHandBlunt => "1-H Blunt",
        WeaponClass::TwoHandBlunt => "2-H Blunt",
        WeaponClass::OneHandSharp => "1-H Sharp",
        WeaponClass::TwoHandSharp => "2-H Sharp",
        WeaponClass::Limited | WeaponClass::Unknown(_) => "Staff",
    }
}

fn armour_word(a: ArmourType) -> &'static str {
    match a {
        ArmourType::Natural | ArmourType::Robes => "Robes",
        ArmourType::Padded => "Padded",
        ArmourType::SoftLeather
        | ArmourType::SoftStuddedLeather
        | ArmourType::RigidLeather
        | ArmourType::StuddedRigidLeather => "Leather",
        ArmourType::Chainmail => "Chain",
        ArmourType::Scalemail => "Scale",
        ArmourType::Platemail | ArmourType::Unknown(_) => "Any",
    }
}

fn magic_word(c: &Class) -> String {
    let school = match c.magery {
        Magery::None | Magery::Unknown(_) => return "None".into(),
        Magery::Mage => "Mage",
        Magery::Priest => "Priest",
        Magery::Druid => "Druid",
        Magery::Bard => return "Bardic".into(),
        Magery::Kai => return "Kai".into(),
    };
    format!("{school}-{}", c.magelvl)
}

/// The class topic's box, as the stock class topics lay it out.
fn class_box(c: &Class, words: &str, text: &str, abilities: &str) -> String {
    let title = words.to_uppercase();
    let exp = format!("EXP: {}%", i32::from(c.exp) + 100);
    let gap = INNER.saturating_sub(2 + title.chars().count() + exp.chars().count());
    let mut out = rule('╔', '╗');
    out.push_str(&row(&format!("{title}{}{exp}", " ".repeat(gap))));
    out.push_str(&rule('╠', '╣'));
    for line in wrap(text, INNER - 2) {
        out.push_str(&row(&line));
    }
    out.push_str(&rule('╠', '╣'));
    let stats = format!(
        "HP/Level: {}-{}  Magic: {}  Combat: {}  Weapons: {}  Armour: {}",
        c.basehp,
        c.basehp + c.hplevel,
        magic_word(c),
        fighting_word(c.combat),
        weapons_word(c.weapon),
        armour_word(c.armour),
    );
    out.push_str(&row(&stats));
    out.push_str(&rule('╠', '╣'));
    out.push_str(&row(&format!("Abilities: {abilities}")));
    out.push_str(&rule('╚', '╝'));
    out
}
