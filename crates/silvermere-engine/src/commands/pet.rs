//! Fork: PET and PET NAME (design/warden.md). A character with a bonded companion sees it
//! with `PET` and names it with `PET NAME <name>`. The line is taken only from a character
//! that has a companion, and only in those two shapes, so `pet <someone>` stays a social
//! action.

use silvermere_text::Msg;
use silvermere_types::fork::{PET_SOURCE_CLASS, Pet, valid_pet_name};
use silvermere_types::ids::SessionId;

use super::parse::{CommandLine, eq_ignore_case};
use crate::combat::pet::{pet_kind, pet_names};
use crate::event_loop::{EngineOutput, PersistRequest};
use crate::tell::tell_user;
use crate::world::World;

fn say(world: &mut World, id: SessionId, text: String) {
    tell_user(world, id, Msg::ForkText { text });
}

/// Handles `line` when it is PET or PET NAME from a character with a companion.
pub fn try_pet(world: &mut World, id: SessionId, line: &CommandLine) -> bool {
    if !eq_ignore_case(line.first(), "pet") {
        return false;
    }
    let naming = line.argc() == 3 && eq_ignore_case(line.arg(1), "name");
    if line.argc() != 1 && !naming {
        return false;
    }
    let Some(kind) = pet_kind(world, id) else {
        return false;
    };
    if !naming {
        let (yours, _) = pet_names(world, id, &kind);
        let blows = match kind.energy_per_blow {
            0..=500 => "several times a round",
            501..=1000 => "once a round",
            _ => "every other round",
        };
        let named = world.pet_names.contains_key(&id);
        let what = if named {
            format!("{yours}, your {},", kind.name)
        } else {
            yours.clone()
        };
        say(
            world,
            id,
            format!("{what} fights at your side, striking {blows}. PET NAME <name> names it."),
        );
        return true;
    }
    let name = line.arg(2).to_string();
    if !valid_pet_name(&name) {
        say(
            world,
            id,
            "A companion's name is one word of letters, up to 10 long.".to_string(),
        );
        return true;
    }
    let mut name = name.to_ascii_lowercase();
    name[..1].make_ascii_uppercase();
    world.pet_names.insert(id, name.clone());
    // saved for the account; a session with none (a test's) keeps it for this visit
    if let Some(account) = world
        .session(id)
        .and_then(|s| s.account.as_ref())
        .map(|a| a.id)
    {
        world
            .outbox
            .push(EngineOutput::Persist(PersistRequest::Pet(Box::new(Pet {
                account,
                source: PET_SOURCE_CLASS.to_string(),
                kind: kind.number,
                name: name.clone(),
                summoned: true,
                immortal: true,
                dead: false,
            }))));
    }
    say(
        world,
        id,
        format!("Your {} is now called {name}.", kind.name),
    );
    true
}
