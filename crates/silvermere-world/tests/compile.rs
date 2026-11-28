//! Compiling builder files: numbers, exits, checks, and a file the content loader takes.

use silvermere_content::Content;
use silvermere_world::model::{Area, AreaHead, Claims, ExitLine, Room};
use silvermere_world::{Used, compile};

fn room(name: &str, at: [i32; 3]) -> Room {
    Room {
        name: name.into(),
        like: "1/2140".into(),
        at,
        title: format!("Newhaven, {name}"),
        desc: "A line.\nAnother line.\n".into(),
        ..Default::default()
    }
}

fn exit(from: &str, dir: &str, to: &str) -> ExitLine {
    ExitLine {
        from: from.into(),
        dir: dir.into(),
        to: to.into(),
        one_way: false,
    }
}

fn area() -> Area {
    Area {
        area: AreaHead {
            name: "Test yard".into(),
            map: 1,
        },
        room: vec![
            Room {
                npc: Some(74),
                max_monsters: 2,
                mon_group: Some(6),
                min_level: Some(1),
                max_level: Some(3),
                ..room("yard", [1, 0, 0])
            },
            room("barn", [2, 0, 0]),
        ],
        exit: vec![exit("1/2140", "east", "yard"), exit("yard", "e", "barn")],
    }
}

/// A world with the stock rooms the area builds on, and 1/9000 taken.
fn used() -> Used {
    let mut u = Used::default();
    u.rooms.extend([(1, 2140), (1, 2141), (1, 9000)]);
    u.exits.insert(((1, 2140), 1), (1, 2141)); // south
    u
}

#[test]
fn new_rooms_get_free_numbers_and_the_file_loads() {
    let c = compile(&[area()], &used(), &Claims::default()).expect("compiles");
    assert_eq!(c.claims.rooms["yard"], "1/9001");
    assert_eq!(c.claims.rooms["barn"], "1/9002");
    let content = Content::from_files(&[("builder.toml", &c.world)]).expect("the loader takes it");
    let w = &content.world;
    assert_eq!(w.room.len(), 2);
    let yard = &w.room[0];
    assert_eq!(yard.room, "1/9001");
    assert_eq!(yard.name, "Newhaven, yard");
    assert_eq!(yard.desc, ["A line.", "Another line."]);
    assert_eq!(yard.specmon, Some(74));
    // nothing of the copied room's NPC, shop or script
    let barn = &w.room[1];
    assert_eq!(
        (barn.specmon, barn.shop, barn.script),
        (Some(0), Some(0), Some(0))
    );
    assert_eq!(yard.max_monsters, Some(2));
    assert_eq!(
        (yard.mon_group, yard.min_level, yard.max_level),
        (Some(6), Some(1), Some(3))
    );
    let exits: Vec<(&str, u32)> = yard
        .exits
        .iter()
        .map(|(d, e)| (d.as_str(), e.to()))
        .collect();
    assert_eq!(exits, [("east", 9002), ("west", 2140)]);
    assert_eq!(w.room_change.len(), 1);
    assert_eq!(w.room_change[0].room, "1/2140");
    assert_eq!(w.room_change[0].exits["east"].to(), 9001);
}

#[test]
fn a_claimed_name_keeps_its_number() {
    let previous = Claims {
        rooms: [("barn".to_string(), "1/9005".to_string())].into(),
    };
    let mut u = used();
    u.rooms.insert((1, 9005)); // installed last time: ours
    let c = compile(&[area()], &u, &previous).expect("compiles");
    assert_eq!(c.claims.rooms["barn"], "1/9005");
    assert_eq!(c.claims.rooms["yard"], "1/9001");
}

#[test]
fn a_name_whose_map_changed_gets_a_new_number() {
    let previous = Claims {
        rooms: [("barn".to_string(), "2/9000".to_string())].into(),
    };
    let c = compile(&[area()], &used(), &previous).expect("compiles");
    assert_eq!(c.claims.rooms["barn"], "1/9002");
    assert!(c.notes.iter().any(|n| n.contains("barn moves from 2/9000")));
}

#[test]
fn an_existing_stock_exit_is_refused() {
    let mut a = area();
    a.exit.push(exit("1/2140", "south", "barn"));
    let e = compile(&[a], &used(), &Claims::default()).expect_err("refused");
    assert!(
        e.0.iter().any(|p| p.contains("already has an exit south")),
        "{e}"
    );
}

#[test]
fn problems_are_all_listed() {
    let mut a = area();
    a.room.push(Room {
        title: String::new(),
        like: "9/1".into(),
        ..room("bad", [0, 0, 0])
    });
    a.exit.push(exit("yard", "east", "nowhere"));
    a.exit.push(exit("yard", "sideways", "barn"));
    let e = compile(&[a], &used(), &Claims::default()).expect_err("refused");
    let all = e.to_string();
    for want in [
        "isn't a room in this world",
        "title",
        "no room named `nowhere`",
        "no direction",
    ] {
        assert!(all.contains(want), "missing `{want}` in:\n{all}");
    }
}

#[test]
fn the_hand_written_world_is_counted_as_used() {
    let mut u = Used::default();
    u.rooms.insert((1, 2140));
    u.add_world_toml(
        "[[room]]\nroom = \"1/9001\"\nlike = \"1/2140\"\nname = \"x\"\ndesc = []\nexits = { west = 2140 }\n\
         [[room_change]]\nroom = \"1/2140\"\nexits = { east = 9001 }\n",
    )
    .expect("reads");
    assert!(u.rooms.contains(&(1, 9001)));
    assert_eq!(u.exits.get(&((1, 2140), 2)), Some(&(1, 9001)));
    let mut a = area();
    a.exit.clear();
    let c = compile(&[a], &u, &Claims::default()).expect("compiles");
    assert_eq!(c.claims.rooms["yard"], "1/9000");
    assert_eq!(c.claims.rooms["barn"], "1/9002");
}

#[test]
fn area_files_round_trip() {
    let a = area();
    let text = a.to_toml().expect("writes");
    assert_eq!(Area::from_toml(&text).expect("reads"), a);
}

#[test]
fn the_compiled_file_loads_beside_the_hand_written_one() {
    let c = compile(&[area()], &used(), &Claims::default()).expect("compiles");
    let world = "[[room]]\nroom = \"1/9000\"\nlike = \"1/2140\"\nname = \"x\"\ndesc = []\n";
    let content = Content::from_files(&[("builder.toml", &c.world), ("world.toml", world)])
        .expect("both load");
    assert_eq!(content.world.room.len(), 3);
    // the same room in both is refused
    let clash = world.replace("9000", "9001");
    assert!(Content::from_files(&[("builder.toml", &c.world), ("world.toml", &clash)]).is_err());
}
