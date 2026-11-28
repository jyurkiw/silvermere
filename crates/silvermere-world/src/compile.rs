//! Compiling builder files into `builder.toml`, in the format silvermere's content loader
//! reads (`silvermere-content`'s `WorldFile`), with numbers allocated against the instance.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use silvermere_types::enums::{Direction, RoomType};
use silvermere_types::names::MONSTER_TYPES;

use crate::model::{Area, Claims, Room};

/// A room: map and room number.
pub type Key = (u32, u32);

/// The first room number (on any map) the compiler gives a new room.
pub const FORK_ROOMS: u32 = 9000;
/// The most characters of a room's name, and of a description line, and the most lines.
pub const TITLE_MAX: usize = 52;
pub const DESC_LINE_MAX: usize = 70;
pub const DESC_LINES: usize = 7;

/// `"1/2140"` as a key.
pub fn parse_key(s: &str) -> Option<Key> {
    let (m, r) = s.split_once('/')?;
    Some((m.trim().parse().ok()?, r.trim().parse().ok()?))
}

pub fn key_str(k: Key) -> String {
    format!("{}/{}", k.0, k.1)
}

/// A direction by name: `north` or `n`, ..., `ne` or `northeast`, `up` or `u`.
pub fn direction(s: &str) -> Option<Direction> {
    Some(match s.trim().to_ascii_lowercase().as_str() {
        "n" | "north" => Direction::North,
        "s" | "south" => Direction::South,
        "e" | "east" => Direction::East,
        "w" | "west" => Direction::West,
        "ne" | "northeast" => Direction::NorthEast,
        "nw" | "northwest" => Direction::NorthWest,
        "se" | "southeast" => Direction::SouthEast,
        "sw" | "southwest" => Direction::SouthWest,
        "u" | "up" => Direction::Up,
        "d" | "down" => Direction::Down,
        _ => return None,
    })
}

/// A direction's full name, as the compiled file writes it.
pub fn dir_name(d: Direction) -> &'static str {
    match d {
        Direction::North => "north",
        Direction::South => "south",
        Direction::East => "east",
        Direction::West => "west",
        Direction::NorthEast => "northeast",
        Direction::NorthWest => "northwest",
        Direction::SouthEast => "southeast",
        Direction::SouthWest => "southwest",
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Unknown(_) => "unknown",
    }
}

/// What the instance already uses: every room, and every stock exit.
#[derive(Debug, Clone, Default)]
pub struct Used {
    pub rooms: BTreeSet<Key>,
    /// A room's exit in a direction (its code) and the room it leads to.
    pub exits: BTreeMap<(Key, u16), Key>,
}

impl Used {
    /// Adds the rooms and exits of a hand-written `world.toml`, which installs beside the
    /// compiled file and may not be in the database yet.
    pub fn add_world_toml(&mut self, text: &str) -> Result<(), String> {
        let t: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        for table in ["room", "room_change"] {
            for r in t
                .get(table)
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                let Some(k) = r.get("room").and_then(|v| v.as_str()).and_then(parse_key) else {
                    continue;
                };
                if table == "room" {
                    self.rooms.insert(k);
                }
                for (d, e) in r
                    .get("exits")
                    .and_then(|v| v.as_table())
                    .into_iter()
                    .flatten()
                {
                    let to = e
                        .as_integer()
                        .or_else(|| e.get("to").and_then(|v| v.as_integer()));
                    if let (Some(d), Some(to)) =
                        (direction(d), to.and_then(|n| u32::try_from(n).ok()))
                    {
                        self.exits.insert((k, d.code()), (k.0, to));
                    }
                }
            }
        }
        Ok(())
    }
}

/// The compiled file, its claims, and notes for the person compiling.
#[derive(Debug, Clone)]
pub struct Compiled {
    /// `builder.toml`.
    pub world: String,
    /// `builder.claims.toml`'s contents.
    pub claims: Claims,
    /// What was numbered, kept, moved or dropped.
    pub notes: Vec<String>,
}

/// Why the builder files didn't compile: every problem found.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{}", .0.join("\n"))]
pub struct Problems(pub Vec<String>);

/// One end of an exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum End {
    /// A room the builder files add, by its index in `rooms`.
    Ours(usize),
    Stock(Key),
}

/// Compiles `areas` against what the instance `used`, keeping the `previous` compile's
/// numbers where they're still free.
pub fn compile(areas: &[Area], used: &Used, previous: &Claims) -> Result<Compiled, Problems> {
    let mut problems = Vec::new();
    let mut notes = Vec::new();

    // the rooms, checked
    let rooms: Vec<(u32, &Room)> = areas
        .iter()
        .flat_map(|a| a.room.iter().map(move |r| (a.area.map, r)))
        .collect();
    let mut by_name: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, (_, r)) in rooms.iter().enumerate() {
        let n = r.name.as_str();
        if n.trim().is_empty() || n.contains('/') || n.trim() != n {
            problems.push(format!("room `{n}`: a name can't be empty or hold a `/`"));
        } else if by_name.insert(n, i).is_some() {
            problems.push(format!("room `{n}`: two rooms have this name"));
        }
        check_room(r, used, &mut problems);
    }

    // numbers: a claimed name keeps its number while nothing else holds it
    let ours_before: BTreeSet<Key> = previous
        .rooms
        .values()
        .filter_map(|s| parse_key(s))
        .collect();
    let foreign = |k: &Key| used.rooms.contains(k) && !ours_before.contains(k);
    let mut number: Vec<Option<Key>> = vec![None; rooms.len()];
    let mut given: BTreeSet<Key> = BTreeSet::new();
    for (i, (map, r)) in rooms.iter().enumerate() {
        let Some(old) = previous.rooms.get(&r.name).and_then(|s| parse_key(s)) else {
            continue;
        };
        if old.0 == *map && !foreign(&old) && !given.contains(&old) {
            number[i] = Some(old);
            given.insert(old);
        } else if old.0 != *map {
            notes.push(format!(
                "{} moves from {} to map {map}: it gets a new number",
                r.name,
                key_str(old)
            ));
        } else {
            notes.push(format!(
                "{} moves from {}: something else in this world holds that number now",
                r.name,
                key_str(old)
            ));
        }
    }
    for (i, (map, r)) in rooms.iter().enumerate() {
        if number[i].is_some() {
            continue;
        }
        let k = (FORK_ROOMS..)
            .map(|n| (*map, n))
            .find(|k| !used.rooms.contains(k) && !given.contains(k) && !ours_before.contains(k))
            .expect("a free room number");
        number[i] = Some(k);
        given.insert(k);
        notes.push(format!("{} is new: {}", r.name, key_str(k)));
    }
    let number: Vec<Key> = number
        .into_iter()
        .map(|k| k.expect("numbered above"))
        .collect();
    for (name, k) in &previous.rooms {
        if !by_name.contains_key(name.as_str()) {
            notes.push(format!(
                "{name} ({k}) is gone; its installed room stays in the database until a re-import"
            ));
        }
    }

    // exits: each side once, both sides unless one way
    let resolve = |s: &str, problems: &mut Vec<String>| -> Option<(End, Key)> {
        if s.contains('/') {
            let k = parse_key(s);
            match k {
                Some(k) if used.rooms.contains(&k) && !ours_before.contains(&k) => {
                    Some((End::Stock(k), k))
                }
                Some(k) if used.rooms.contains(&k) => {
                    problems.push(format!(
                        "exit: `{s}` is one of the editor's own rooms; name it instead"
                    ));
                    None
                }
                _ => {
                    problems.push(format!("exit: no room `{s}` in this world"));
                    None
                }
            }
        } else {
            match by_name.get(s) {
                Some(&i) => Some((End::Ours(i), number[i])),
                None => {
                    problems.push(format!("exit: no room named `{s}`"));
                    None
                }
            }
        }
    };
    let mut sides: BTreeMap<(End, u16), (Direction, End, Key)> = BTreeMap::new();
    for a in areas {
        for x in &a.exit {
            let what = format!("exit {} {} {}", x.from, x.dir, x.to);
            let (Some((from, fk)), Some((to, tk))) = (
                resolve(&x.from, &mut problems),
                resolve(&x.to, &mut problems),
            ) else {
                continue;
            };
            let Some(d) = direction(&x.dir) else {
                problems.push(format!("{what}: no direction `{}`", x.dir));
                continue;
            };
            if fk.0 != tk.0 {
                problems.push(format!("{what}: an exit can't leave its map"));
                continue;
            }
            let mut put = |from: End, fk: Key, d: Direction, to: End, tk: Key| {
                if let End::Stock(k) = from
                    && let Some(&there) = used.exits.get(&(k, d.code()))
                    && !ours_before.contains(&there)
                {
                    problems.push(format!(
                        "{what}: {} already has an exit {} (to {}); changing stock exits isn't supported yet",
                        key_str(k),
                        dir_name(d),
                        key_str(there)
                    ));
                    return;
                }
                if sides.insert((from, d.code()), (d, to, tk)).is_some() {
                    problems.push(format!(
                        "{what}: {} has two exits {}",
                        key_str(fk),
                        dir_name(d)
                    ));
                }
            };
            put(from, fk, d, to, tk);
            if !x.one_way {
                match d.reverse() {
                    Some(back) => put(to, tk, back, from, fk),
                    None => problems.push(format!("{what}: no way back")),
                }
            }
        }
    }
    if !problems.is_empty() {
        return Err(Problems(problems));
    }

    // the file
    let mut out = format!(
        "# Compiled by silvermere-world {} from the world editor's builder files\n\
         # (MMUD_Builder, design/world-editor.md). Don't edit: change the builder files and\n\
         # compile again.\n",
        env!("CARGO_PKG_VERSION")
    );
    let exits_of = |end: End| -> Vec<(Direction, u32)> {
        sides
            .range((end, 0)..=(end, u16::MAX))
            .map(|(_, (d, _, tk))| (*d, tk.1))
            .collect()
    };
    let stock_from: BTreeSet<Key> = sides
        .keys()
        .filter_map(|(e, _)| match e {
            End::Stock(k) => Some(*k),
            End::Ours(_) => None,
        })
        .collect();
    for k in stock_from {
        let _ = write!(
            out,
            "\n[[room_change]]\nroom = {}\nexits = {}\n",
            q(&key_str(k)),
            inline_exits(&exits_of(End::Stock(k)))
        );
    }
    for (i, (_, r)) in rooms.iter().enumerate() {
        let _ = write!(
            out,
            "\n# {}\n[[room]]\nroom = {}\nlike = {}\nname = {}\n",
            r.name,
            q(&key_str(number[i])),
            q(&r.like),
            q(&r.title)
        );
        out.push_str("desc = [\n");
        for line in desc_lines(&r.desc) {
            let _ = writeln!(out, "  {},", q(line));
        }
        out.push_str("]\n");
        let _ = writeln!(out, "exits = {}", inline_exits(&exits_of(End::Ours(i))));
        if let Some(k) = &r.kind {
            let _ = writeln!(out, "kind = {}", q(k));
        }
        // a new room gets none of the copied room's NPC, shop or script: those belong to the
        // stock room, and copying one by surprise doubles an NPC or a shop
        let _ = writeln!(out, "specmon = {}", r.npc.unwrap_or(0));
        out.push_str("shop = 0\nscript = 0\n");
        let _ = writeln!(out, "max_monsters = {}", r.max_monsters);
        for (field, v) in [
            ("mon_group", r.mon_group.map(i32::from)),
            ("min_level", r.min_level.map(i32::from)),
            ("max_level", r.max_level.map(i32::from)),
            ("gen_monster", r.gen_monster),
        ] {
            if let Some(v) = v {
                let _ = writeln!(out, "{field} = {v}");
            }
        }
    }
    let claims = Claims {
        rooms: rooms
            .iter()
            .enumerate()
            .map(|(i, (_, r))| (r.name.clone(), key_str(number[i])))
            .collect(),
    };
    Ok(Compiled {
        world: out,
        claims,
        notes,
    })
}

fn check_room(r: &Room, used: &Used, problems: &mut Vec<String>) {
    let n = &r.name;
    match parse_key(&r.like) {
        Some(k) if used.rooms.contains(&k) => {}
        _ => problems.push(format!(
            "room `{n}`: like `{}` isn't a room in this world",
            r.like
        )),
    }
    if r.title.trim().is_empty() || r.title.chars().count() > TITLE_MAX {
        problems.push(format!(
            "room `{n}`: the title needs 1 to {TITLE_MAX} characters"
        ));
    }
    let lines = desc_lines(&r.desc);
    if lines.len() > DESC_LINES {
        problems.push(format!(
            "room `{n}`: at most {DESC_LINES} description lines"
        ));
    }
    if let Some(l) = lines.iter().find(|l| l.chars().count() > DESC_LINE_MAX) {
        problems.push(format!(
            "room `{n}`: a description line is longer than {DESC_LINE_MAX} characters: `{l}`"
        ));
    }
    if let Some(k) = &r.kind
        && !RoomType::ALL
            .iter()
            .any(|t| format!("{t:?}").eq_ignore_ascii_case(k))
    {
        problems.push(format!("room `{n}`: no room kind `{k}`"));
    }
    if let Some(g) = r.mon_group
        && usize::try_from(g)
            .ok()
            .is_none_or(|g| g >= MONSTER_TYPES.len())
    {
        problems.push(format!("room `{n}`: no monster group {g}"));
    }
}

/// A description's lines, without the trailing blank ones.
pub fn desc_lines(desc: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = desc.lines().map(str::trim_end).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

fn q(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

fn inline_exits(exits: &[(Direction, u32)]) -> String {
    if exits.is_empty() {
        return "{}".into();
    }
    let parts: Vec<String> = exits
        .iter()
        .map(|(d, n)| format!("{} = {n}", dir_name(*d)))
        .collect();
    format!("{{ {} }}", parts.join(", "))
}
