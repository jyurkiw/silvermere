//! New rooms, monsters, shops, items and text blocks, and changes to stock rooms and items
//! (`content/world.toml`; design/warden.md, the Newhaven stable).
//!
//! A new record copies a stock record (`like`), read from the database at install, and
//! overrides what the content names. New records live in fork ranges far above stock's
//! numbers ([`FORK_ROOMS`] and the rest), so a record found there is the content's own and is
//! overwritten; nothing else is.

use silvermere_db::Db;
use silvermere_db::repo::StateWrite;
use silvermere_types::enums::{Ability, Direction, ExitType, RoomType};
use silvermere_types::ids::{
    ActiveMonsterId, ItemNo, MonsterNo, RoomId, RoomKey, ShopNo, TextBlockNo,
};
use silvermere_types::records::{AbilitySlot, Coins, Exit, RoomItem, ShopSlot, TextBlock};

use crate::install::InstallError;
use crate::model::{AbilityEntry, WorldFile};

/// The first room number (on any map) of the fork's rooms.
pub const FORK_ROOMS: u32 = 9000;
/// The first monster number of the fork's monsters.
pub const FORK_MONSTERS: i32 = 9000;
/// The first item number of the fork's items.
pub const FORK_ITEMS: i32 = 9000;
/// The first shop number of the fork's shops.
pub const FORK_SHOPS: i32 = 900;
/// The first text block number of the fork's text blocks.
pub const FORK_TEXTS: i32 = 20000;

/// `"1/9000"` as a room key.
pub fn room_key(s: &str) -> Option<RoomKey> {
    let (m, r) = s.split_once('/')?;
    Some(RoomKey::new(m.trim().parse().ok()?, r.trim().parse().ok()?))
}

/// A direction by its name: `north` or `n`, ..., `ne` or `northeast`, `up` or `u`.
pub fn direction(s: &str) -> Option<Direction> {
    Some(match s.to_ascii_lowercase().as_str() {
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

fn ability(name: &str) -> Option<Ability> {
    silvermere_types::fork::fork_ability(name).or_else(|| {
        Ability::ALL
            .iter()
            .copied()
            .find(|a| format!("{a:?}").eq_ignore_ascii_case(name))
    })
}

fn abilities(list: &[AbilityEntry]) -> Result<Vec<AbilitySlot>, String> {
    list.iter()
        .map(|a| {
            ability(&a.name)
                .map(|x| (x, a.value))
                .ok_or_else(|| format!("unknown ability `{}`", a.name))
        })
        .collect()
}

fn set_abilities(slots: &mut [AbilitySlot], add: &[AbilitySlot]) -> Result<(), String> {
    for &(a, v) in add {
        if let Some(s) = slots.iter_mut().find(|s| s.0 == a) {
            s.1 = v;
        } else if let Some(s) = slots.iter_mut().find(|s| s.0 == Ability::None) {
            *s = (a, v);
        } else {
            return Err("no free ability slot".into());
        }
    }
    Ok(())
}

fn lines<const N: usize>(text: &[String]) -> [String; N] {
    std::array::from_fn(|i| text.get(i).cloned().unwrap_or_default())
}

fn exits(list: &std::collections::BTreeMap<String, u32>) -> Result<Vec<(usize, u32)>, String> {
    list.iter()
        .map(|(d, to)| {
            direction(d)
                .map(|d| (usize::from(d.code()), *to))
                .ok_or_else(|| format!("unknown direction `{d}`"))
        })
        .collect()
}

fn exit_to(target: u32) -> Exit {
    Exit {
        target: RoomId(target),
        kind: ExitType::from_code(0),
        par1: 0,
        par2: 0,
        par3: 0,
        par4: 0,
    }
}

/// Checks the world file without a database: names, ranges, sizes.
pub fn check(w: &WorldFile) -> Result<(), String> {
    for r in &w.room {
        let key =
            room_key(&r.room).ok_or_else(|| format!("room `{}` is not like 1/9000", r.room))?;
        if key.room.0 < FORK_ROOMS {
            return Err(format!(
                "room {}: new rooms are numbered from {FORK_ROOMS}",
                r.room
            ));
        }
        room_key(&r.like)
            .ok_or_else(|| format!("room {}: like `{}` is not a room", r.room, r.like))?;
        if r.desc.len() > 7 {
            return Err(format!("room {}: at most 7 description lines", r.room));
        }
        exits(&r.exits).map_err(|e| format!("room {}: {e}", r.room))?;
        if let Some(k) = &r.kind {
            RoomType::ALL
                .iter()
                .find(|t| format!("{t:?}").eq_ignore_ascii_case(k))
                .ok_or_else(|| format!("room {}: unknown kind `{k}`", r.room))?;
        }
    }
    for c in &w.room_change {
        room_key(&c.room).ok_or_else(|| format!("room change `{}` is not a room", c.room))?;
        exits(&c.exits).map_err(|e| format!("room change {}: {e}", c.room))?;
    }
    for m in &w.monster {
        if m.number < FORK_MONSTERS || m.desc.len() > 4 {
            return Err(format!(
                "monster {}: numbered from {FORK_MONSTERS}, at most 4 lines",
                m.number
            ));
        }
    }
    for s in &w.shop {
        if s.number < FORK_SHOPS || s.items.len() > silvermere_types::sizes::NSHOPITEMS {
            return Err(format!(
                "shop {}: numbered from {FORK_SHOPS}, at most 20 items",
                s.number
            ));
        }
    }
    for i in &w.item {
        if i.number < FORK_ITEMS || i.desc.len() > 6 {
            return Err(format!(
                "item {}: numbered from {FORK_ITEMS}, at most 6 lines",
                i.number
            ));
        }
        abilities(&i.abilities).map_err(|e| format!("item {}: {e}", i.number))?;
    }
    for c in &w.item_change {
        abilities(&c.add_abilities).map_err(|e| format!("item change {}: {e}", c.number))?;
    }
    for t in &w.text {
        if t.number < FORK_TEXTS || t.text.len() > 1999 {
            return Err(format!(
                "text {}: numbered from {FORK_TEXTS}, at most 1999 bytes",
                t.number
            ));
        }
    }
    Ok(())
}

/// A record whose stock template isn't in this world (a test's small world, a realm that
/// isn't the stock one) is left out, with a warning, rather than stopping the start.
fn skipped(what: String) {
    tracing::warn!("content: {what} is not in this world; the world content is left out");
}

fn bad(e: String) -> InstallError {
    InstallError::Change(e)
}

/// The first stock record the world file copies or changes that `db` doesn't have.
async fn first_missing(db: &Db, w: &WorldFile) -> Result<Option<String>, InstallError> {
    for i in &w.item {
        if db.items().get(ItemNo(i.like)).await?.is_none() {
            return Ok(Some(format!("item {}", i.like)));
        }
    }
    for c in &w.item_change {
        if db.items().get(ItemNo(c.number)).await?.is_none() {
            return Ok(Some(format!("item {}", c.number)));
        }
    }
    for m in &w.monster {
        if db.monsters().get(MonsterNo(m.like)).await?.is_none() {
            return Ok(Some(format!("monster {}", m.like)));
        }
    }
    for s in &w.shop {
        if db.shops().get(ShopNo(s.like)).await?.is_none() {
            return Ok(Some(format!("shop {}", s.like)));
        }
    }
    for (what, key) in w
        .room
        .iter()
        .map(|r| &r.like)
        .chain(w.room_change.iter().map(|c| &c.room))
        .map(|k| (k, room_key(k)))
    {
        let Some(key) = key else {
            return Ok(Some(format!("room {what}")));
        };
        if db.rooms().get(key).await?.is_none() {
            return Ok(Some(format!("room {what}")));
        }
    }
    Ok(None)
}

/// Writes the world file into `db`: text blocks, items, monsters, shops, new rooms, then the
/// changes to stock rooms and items. Returns how many records it wrote.
pub async fn install_world(db: &Db, w: &WorldFile) -> Result<usize, InstallError> {
    // all or nothing: a world without every stock record the file builds on (a test's small
    // world, a realm that isn't the stock one) gets none of it, rather than half a stable
    if let Some(what) = first_missing(db, w).await? {
        skipped(what);
        return Ok(0);
    }
    let mut n = 0;
    for t in &w.text {
        let mut text = t.text.clone();
        if !text.ends_with('\n') {
            text.push('\n');
        }
        db.text_blocks()
            .put(&TextBlock {
                part: 0,
                number: TextBlockNo(t.number),
                link: TextBlockNo(t.link),
                text,
            })
            .await?;
        n += 1;
    }
    for i in &w.item {
        let Some(mut item) = db.items().get(ItemNo(i.like)).await? else {
            skipped(format!("item {}", i.like));
            continue;
        };
        item.number = ItemNo(i.number);
        item.name = i.name.clone();
        item.desc = lines(&i.desc);
        item.cost = i.cost;
        set_abilities(&mut item.abilities, &abilities(&i.abilities).map_err(bad)?).map_err(bad)?;
        db.items()
            .put_many(&[item], StateWrite::CreateMissing)
            .await?;
        n += 1;
    }
    for c in &w.item_change {
        let Some(mut item) = db.items().get(ItemNo(c.number)).await? else {
            skipped(format!("item {}", c.number));
            continue;
        };
        set_abilities(
            &mut item.abilities,
            &abilities(&c.add_abilities).map_err(bad)?,
        )
        .map_err(bad)?;
        db.items().put_many(&[item], StateWrite::Untouched).await?;
        n += 1;
    }
    for m in &w.monster {
        let Some(mut mon) = db.monsters().get(MonsterNo(m.like)).await? else {
            skipped(format!("monster {}", m.like));
            continue;
        };
        mon.number = MonsterNo(m.number);
        mon.name = m.name.clone();
        mon.desc = lines(&m.desc);
        if let Some(t) = m.talk {
            mon.talktext = TextBlockNo(t);
        }
        db.monsters()
            .put_many(&[mon], StateWrite::CreateMissing)
            .await?;
        n += 1;
    }
    for s in &w.shop {
        let Some(mut shop) = db.shops().get(ShopNo(s.like)).await? else {
            skipped(format!("shop {}", s.like));
            continue;
        };
        shop.number = ShopNo(s.number);
        shop.name = s.name.clone();
        for (i, slot) in shop.stock.iter_mut().enumerate() {
            *slot = match s.items.get(i) {
                Some(&item) => ShopSlot {
                    item: ItemNo(item),
                    maxqty: 99,
                    qty: 99,
                    regentime: 1,
                    regennum: 99,
                    regenpct: 100,
                },
                None => ShopSlot {
                    item: ItemNo(0),
                    maxqty: 0,
                    qty: 0,
                    regentime: 0,
                    regennum: 0,
                    regenpct: 0,
                },
            };
        }
        db.shops()
            .put_many(&[shop], StateWrite::CreateMissing)
            .await?;
        n += 1;
    }
    for r in &w.room {
        let key = room_key(&r.room).ok_or_else(|| bad(format!("room `{}`", r.room)))?;
        let like = room_key(&r.like).ok_or_else(|| bad(format!("room `{}`", r.like)))?;
        let Some(mut room) = db.rooms().get(like).await? else {
            skipped(format!("room {}", r.like));
            continue;
        };
        room.key = key;
        // a new room starts empty: none of the copied room's floor, coins or monsters
        let empty = RoomItem {
            item: ItemNo(0),
            uses: 0,
            count: 0,
        };
        room.items = std::array::from_fn(|_| empty);
        room.hiditems = std::array::from_fn(|_| empty);
        room.placed = std::array::from_fn(|_| ItemNo(0));
        room.monsters = std::array::from_fn(|_| ActiveMonsterId(0));
        room.coins = Coins::default();
        room.hidcoins = Coins::default();
        room.genmonster = MonsterNo(0);
        room.name = r.name.clone();
        room.desc = lines(&r.desc);
        for e in room.exits.iter_mut() {
            *e = exit_to(0);
        }
        for (d, to) in exits(&r.exits).map_err(bad)? {
            room.exits[d] = exit_to(to);
        }
        if let Some(k) = &r.kind {
            room.room_type = RoomType::ALL
                .iter()
                .copied()
                .find(|t| format!("{t:?}").eq_ignore_ascii_case(k))
                .unwrap_or(room.room_type);
        }
        if let Some(s) = r.shop {
            room.shopnum = ShopNo(s);
        }
        if let Some(m) = r.specmon {
            room.specmon = MonsterNo(m);
        }
        if let Some(t) = r.script {
            room.cmdtext = TextBlockNo(t);
        }
        if let Some(m) = r.max_monsters {
            room.maxmonsters = m;
        }
        db.rooms()
            .put_many(&[room], StateWrite::CreateMissing)
            .await?;
        n += 1;
    }
    for c in &w.room_change {
        let key = room_key(&c.room).ok_or_else(|| bad(format!("room `{}`", c.room)))?;
        let Some(mut room) = db.rooms().get(key).await? else {
            skipped(format!("room {}", c.room));
            continue;
        };
        for (d, to) in exits(&c.exits).map_err(bad)? {
            room.exits[d] = exit_to(to);
        }
        db.rooms().put_many(&[room], StateWrite::Untouched).await?;
        n += 1;
    }
    Ok(n)
}
