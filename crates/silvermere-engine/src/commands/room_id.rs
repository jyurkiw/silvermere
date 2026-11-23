//! Fork: `/room` tells anyone where they are, as `map/room` (`1/101`), for mapping and
//! scripting. `/room <direction>` (`n`, `s`, `e`, `w`, `ne`, `nw`, `se`, `sw`, `u`, `d`)
//! tells the room an exit that way leads to, or `*no room found*` when the room shows no
//! exit there. It knows only what `Obvious exits:` shows: a hidden exit not yet found, a
//! remote action and a text exit (`go portal`) count as none, so nothing secret leaks.
//!
//! Only `/room` alone or with one of those directions is taken: `/room <other text>` is
//! still a telepath to a player called Room, as stock's `/name text`.

use silvermere_text::Msg;
use silvermere_types::ids::{RoomKey, SessionId};

use super::parse::{CommandLine, eq_ignore_case};
use crate::combat::player;
use crate::display::exits::visible_exit;
use crate::movement::exits::live_exits;
use crate::tell::tell_user;
use crate::world::World;

/// The direction words, in exit order (north, south, east, west, northeast, northwest,
/// southeast, southwest, up, down).
const DIRECTIONS: [&str; 10] = ["n", "s", "e", "w", "ne", "nw", "se", "sw", "u", "d"];

/// What `/room <direction>` answers when the room shows no exit that way.
pub const NO_ROOM: &str = "*no room found*";

/// Handles `line` when it is `/room`, alone or with a direction.
pub fn try_room_id(world: &mut World, id: SessionId, line: &CommandLine) -> bool {
    if !eq_ignore_case(line.first(), "/room") {
        return false;
    }
    let dir = match line.argc() {
        1 => None,
        2 => match DIRECTIONS.iter().position(|d| eq_ignore_case(line.arg(1), d)) {
            Some(d) => Some(d),
            // `/room hello` is a telepath
            None => return false,
        },
        _ => return false,
    };
    let Some(plr) = player(world, id) else {
        return false;
    };
    let here = RoomKey::new(plr.map.0, plr.room.0);
    let text = match dir {
        None => format!("{}/{}", here.map.0, here.room.0),
        Some(d) => live_exits(world, here)
            .and_then(|exits| exits.get(d).copied())
            .filter(visible_exit)
            .map_or_else(
                || NO_ROOM.to_owned(),
                |e| format!("{}/{}", here.map.0, e.target.0),
            ),
    };
    tell_user(world, id, Msg::ForkText { text });
    true
}
