//! Fork: `/room` tells anyone where they are, as `map/room` (`1/101`), for mapping and
//! scripting. Only the line `/room` alone is taken: `/room <text>` is still a telepath to a
//! player called Room, as stock's `/name text`.

use silvermere_text::Msg;
use silvermere_types::ids::SessionId;

use super::parse::{CommandLine, eq_ignore_case};
use crate::combat::player;
use crate::tell::tell_user;
use crate::world::World;

/// Handles `line` when it is `/room` alone.
pub fn try_room_id(world: &mut World, id: SessionId, line: &CommandLine) -> bool {
    if line.argc() != 1 || !eq_ignore_case(line.first(), "/room") {
        return false;
    }
    let Some(plr) = player(world, id) else {
        return false;
    };
    let text = format!("{}/{}", plr.map.0, plr.room.0);
    tell_user(world, id, Msg::ForkText { text });
    true
}
