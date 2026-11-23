//! Fork: `/room` tells anyone the room they are in, as `map/room`.

use crate::common::{enter, tiny_snapshot, test_config};
use silvermere_engine::commands::input::execute_input;
use silvermere_engine::world::World;
use silvermere_text::strip_ansi;
use silvermere_types::enums::MessageLevel;

fn said(w: &mut World, id: silvermere_engine::SessionId, line: &str) -> String {
    let palette = silvermere_engine::tell::palette_of(w, id);
    let prompt = silvermere_engine::tell::prompt_bytes(w, id);
    w.session_mut(id).unwrap().output.clear();
    execute_input(w, id, line, false);
    let s = w.session_mut(id).unwrap();
    let b = s.output.render(palette, &prompt);
    s.output.clear();
    String::from_utf8_lossy(&strip_ansi(&b)).into_owned()
}

#[test]
fn slash_room_tells_the_map_and_room() {
    let mut w = World::load(test_config(), 8, tiny_snapshot()).unwrap();
    let id = enter(&mut w, "Gorin", 2140, MessageLevel::Low);
    let out = said(&mut w, id, "/room");
    assert!(out.contains("1/2140"), "{out:?}");
    let out = said(&mut w, id, "/ROOM");
    assert!(out.contains("1/2140"), "any case: {out:?}");
}

#[test]
fn slash_room_with_text_is_still_a_telepath() {
    let mut w = World::load(test_config(), 8, tiny_snapshot()).unwrap();
    let id = enter(&mut w, "Gorin", 2140, MessageLevel::Low);
    let out = said(&mut w, id, "/room hello there");
    assert!(!out.contains("1/2140"), "{out:?}");
}

/// Gorin in 1/2140, its exits: north a plain exit to 2141, northeast a hidden exit not yet
/// found, southeast a remote action, up a text exit (`go portal`).
fn exits_world() -> (World, silvermere_engine::SessionId) {
    use silvermere_types::enums::ExitType;
    use silvermere_types::ids::{RoomId, RoomKey};
    let mut w = World::load(test_config(), 8, tiny_snapshot()).unwrap();
    let r = w.rooms.get_mut(&RoomKey::new(1, 2140)).unwrap();
    for e in r.exits.iter_mut() {
        e.target = RoomId(0);
        e.kind = ExitType::Normal;
    }
    let set = |e: &mut silvermere_types::records::Exit, to: u32, kind: ExitType| {
        e.target = RoomId(to);
        e.kind = kind;
        (e.par1, e.par2, e.par3, e.par4) = (0, 0, 0, 0);
    };
    set(&mut r.exits[0], 2141, ExitType::Normal);
    set(&mut r.exits[4], 2142, ExitType::Hidden);
    r.exits[4].par1 = silvermere_types::flags::ExitFlags::NEEDSEARCH.bits();
    set(&mut r.exits[6], 2140, ExitType::RemoteAction);
    set(&mut r.exits[8], 2143, ExitType::Text);
    let id = enter(&mut w, "Gorin", 2140, MessageLevel::Low);
    (w, id)
}

#[test]
fn slash_room_with_a_direction_tells_the_room_that_way() {
    let (mut w, id) = exits_world();
    let out = said(&mut w, id, "/room n");
    assert!(out.contains("1/2141"), "{out:?}");
    let out = said(&mut w, id, "/room N");
    assert!(out.contains("1/2141"), "any case: {out:?}");
    let out = said(&mut w, id, "/room w");
    assert!(out.contains("*no room found*"), "no exit: {out:?}");
}

#[test]
fn slash_room_knows_only_the_exits_the_room_shows() {
    let (mut w, id) = exits_world();
    for (dir, what) in [("ne", "a hidden exit not found"), ("se", "a remote action"), ("u", "a text exit")] {
        let out = said(&mut w, id, &format!("/room {dir}"));
        assert!(out.contains("*no room found*"), "{what}: {out:?}");
    }
}

#[test]
fn slash_room_with_other_words_is_still_a_telepath() {
    let (mut w, id) = exits_world();
    for line in ["/room north please", "/room n now"] {
        let out = said(&mut w, id, line);
        assert!(!out.contains("1/2141") && !out.contains("no room found"), "{line}: {out:?}");
    }
}
