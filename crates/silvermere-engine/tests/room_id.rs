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
