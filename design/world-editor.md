# World editor (MMUD_Builder)

Status: **design settled, not implemented.** A read-only 3D map prototype exists
(`tools/world-editor-proto` on branch `proto/world-editor-3d`); nothing else is built.
Fork-only.

- [Summary](#summary)
- [Why it's needed](#why-its-needed)
- [The pieces and where they live](#the-pieces-and-where-they-live)
- [The builder format](#the-builder-format)
- [Compiling](#compiling)
- [Installing: claims and the comb](#installing-claims-and-the-comb)
- [Editing stock records](#editing-stock-records)
- [The app](#the-app)
  - [The 3D map](#the-3d-map)
  - [Rooms: navigation](#rooms-navigation)
  - [Rooms: interaction](#rooms-interaction)
  - [Scripts and talk](#scripts-and-talk)
  - [The other editors](#the-other-editors)
- [Checks](#checks)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Implementation order](#implementation-order)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

The fork's world content (rooms, NPCs, shops, items, texts and scripts) is written today by
hand in `content/world.toml`, with every room, monster, item, shop, text and message
numbered by hand, and exits' parameters written as `par = [20000, 20, 20001, 0]`. The
world editor replaces that with:

- **MMUD_Builder**, a public repo of its own: a native desktop app (Rust, Bevy with egui
  panels) with a **3D map** of rooms and exits, forms for every record, and the builder's
  own source files.
- **A builder format** shaped for building rather than for the game: names instead of
  numbers, exits declared once for both sides, typed exits with named fields, texts
  written where they're used, and editor-only positions.
- **A compiler** that turns the builder format into `content/world.toml` exactly as
  silvermere reads it today, giving every name a number that is **free in the instance it
  will be installed into**.
- **Claims and the comb** in silvermere: the install records which numbers the content
  holds, and when a number has to move, moves the characters, items and monsters that used
  it, or refuses to start until told to.
- **Stock edits**: stock rooms, NPCs, monsters, items and spells can be changed or deleted
  as well, as patches the installer applies over the stock records at every start.

The content overlay ([content-overlay.md](content-overlay.md)) is unchanged in kind: the
compiled file is built into the program and installed at every start.

## Why it's needed

- **Numbers.** Every record is numbered by hand in the fork ranges (rooms 9000+, monsters
  9000+, items 9000+, shops 900+, texts and messages 20000+). Picking them, keeping them
  apart and following them through a script (`takeitem 9000 0:text 20037`) is the slowest
  and most error-prone part of building. Map and room numbers colliding across add-ons is
  a known MajorMUD problem.
- **Exits.** Both sides of every exit are written separately, and a typed exit is four bare
  numbers whose meaning depends on its kind.
- **Seeing it.** There is no picture of an area. MajorMUD areas use up and down heavily
  (towers, the Crypt, stairwells, Newhaven's Arena below the Narrow Road), and how rooms,
  levels, levers and doors relate is invisible in TOML.
- **Gotchas.** New rooms need `max_monsters = 0`, script lines need a `:`, Crypt rooms are
  dark, texts split at 1999 bytes. An editor can check these instead of a person
  remembering them.

## The pieces and where they live

| Piece | Repo | Why there |
|---|---|---|
| The app: 3D map, forms, dig, layout cache | **MMUD_Builder** (public, new) | A desktop app with a heavy dependency tree (Bevy) that the game's build never needs |
| The builder source files (areas, NPCs, items, texts) | **MMUD_Builder** | The builder owns its own format; its CI compiles every push |
| The compiler crate, `silvermere-world` (library and CLI) | **silvermere**, main workspace | It must write exactly the format this silvermere reads; CI tests it against the installer |
| The script-parser crate, split out of `silvermere-engine/src/scripts` | **silvermere**, main workspace | The engine and the editor's script checks share one parser, so they can't disagree |
| Claims, the comb, `content claims export`, `--allow-remap` | **silvermere** | Server side: install time |
| The compiled `content/world.toml` and its claims sidecar | **silvermere** `content/` | Built into the program as today |

MMUD_Builder depends on the silvermere crates (`silvermere-types`, `silvermere-world`, the
script parser) through a git dependency pinned to a silvermere commit. That pin decides
the `world.toml` format the editor writes, so the compiler stamps its version in the file
(see [Compiling](#compiling)).

**Being public.** The builder repo is public, so its sources are too: room text, scripts and
quest solutions (keys, passwords, the tomb quests). Content that should stay a secret
doesn't belong there ([open question 1](#open-questions)).

## The builder format

TOML, one file per area (`areas/newhaven-stable.toml`), plus files for things that aren't
in one area (items, NPCs used in several places). Everything the fork adds is named by a
**name**, unique per kind (`stable-yard`, `old-bannon`, `hounds-leash`). Stock records are
still named by number (`"1/2140"`, `like = 74`), because they're the instance's, not ours.

```toml
[area]
name = "Newhaven stable"
map = 1                          # the map its rooms are on (exits never leave a map)

[[room]]
name = "stable-yard"
like = "1/2140"
at = [1, 0, 0]                   # editor-only position: x east, y up, z south
title = "Newhaven, Stable Yard"
desc = """
A wide yard of packed earth opens east of the village gate, fenced with
...
"""
max_monsters = 0                 # required: a new room has no monster group
npc = "old-bannon"

# both sides at once; one_way = true for a single side
[[exit]]
from = "1/2140"                  # a stock room: the compiler writes a [[room_change]]
dir = "east"
to = "stable-yard"

[[exit]]
from = "stable-yard"
dir = "east"
to = "warden-stables"

# a typed exit with named fields instead of par = [...]
[[exit]]
from = "tomb-passage"
dir = "north"
to = "tomb-hall"
kind = "Key"
key = "bone-key"                 # an item by name
lock = "Locked"
open_steps = 2
```

- **Exits once.** An `[[exit]]` is two-way unless it says `one_way = true`; the compiler
  writes the reverse side with the reverse direction. A one-way exit is a choice, not a
  forgotten line.
- **Typed exits by name.** Each kind's fields are the names `silvermere-types` already
  gives them in `ExitParams` (`KeyExit { key, lock, pick_bonus, open_steps }`,
  `RemoteExit { words, code, message, item }`, ...). Items, messages, spells, classes and
  abilities are written by name; the compiler writes the numbers and the `par` order.
- **Texts where they're used.** A room's script, an NPC's talk and an exit's messages are
  written inline. The compiler gives each its own text block or message number, links
  talk blocks, and splits long blocks at 1999 bytes.
- **Positions are the editor's.** `at` places a room on the editor's grid and is dropped by
  the compiler. It has nothing to do with the room's number: moving a room in the editor
  never changes its number, and its number never changes where it sits.

A **stock edit** names the stock record by number and holds only what changes
([Editing stock records](#editing-stock-records)):

```toml
[[edit]]
room = "1/2146"                  # Newhaven, Narrow Road
title = "Newhaven, Muddy Road"
max_monsters = 2

[[edit]]
monster = 76                     # Dathalar, the General Store's keeper
talk = "dathalar-talk"

[[delete]]
item = 312
```

The first version of the format covers what `world.toml` covers today. It grows with the
game-side format, one record kind at a time ([The other editors](#the-other-editors)).

## Compiling

```
builder files ──► pre-compile: the instance's used numbers ──► allocate ──► world.toml
                    (/v1/backup, the DB read-only, or claims.lock)          + world.claims.toml
```

1. **Pre-compile.** The compiler gets every number in use in the instance it compiles for,
   per kind (rooms per map, monsters, items, shops, texts, messages), and the content's own
   current claims. It gets them from one of:
   - **the instance**: `GET /v1/backup` from a running server (read scope), or the
     database file opened read-only when the server is stopped;
   - **`claims.lock`**: a snapshot exported from the instance by `silvermere content claims
     export`, committed in MMUD_Builder, for compiling where the instance can't be reached
     (GitHub). It holds the content's claims (name to number) and every other number in use,
     as ranges.
2. **Allocate.** A name already claimed keeps its number while that number is still free
   of anything else. A new name, or one whose number something else now holds, gets the
   lowest free number in the fork range for its kind, and above it when the range is full.
   Our content installs last ([Decision 6](#decisions)), so free in the instance is all that
   matters.
3. **Write.**
   - `content/world.toml`, in **exactly the format silvermere reads today**: numbers only.
     The builder name goes in a comment above each record (`# stable-yard`), and the file
     starts with `# compiled by silvermere-world <version> — don't edit`.
   - `content/world.claims.toml`, the sidecar: name to number per kind, for the installer.
     The content loader skips it rather than refusing it as an unknown file.

`silvermere-world build --against <db | url | claims.lock>` is the CLI; the app's Compile
button calls the same library.

## Installing: claims and the comb

The installer stays the one in `silvermere-content`, at every server start, with these
additions. It all runs in one transaction before the world loads.

1. **Collision check.** Every number in `world.toml` must be free or already the content's,
   by the `content_claims` table. A number held by anything else (a re-import, a stock
   update) stops the install: "1/9003 is held by something else in this world; recompile
   against it". The installer never renumbers a compiled file itself.
2. **Remap check.** The sidecar is compared with `content_claims` by name. A name whose
   number changed is a **remap**; a claimed name that's gone is an **orphan**.
3. **The comb.** For each remap and orphan, every live use of the old number is moved:

   | Number | Live uses |
   |---|---|
   | Rooms | `characters.map/room`, `active_monsters.map/room` and `genroom`, `active_monster_trail`, `room_state`, `room_floor_items`, `room_monsters` |
   | Items | `character_inventory`, `characters.weapon`, `character_worn`, `character_keys`, `room_floor_items`, `active_monster_items`, `shop_stock` |
   | Monsters | `active_monsters.knownnum`, `monster_state` |
   | Shops | `shop_stock` |
   | Texts, messages | none (content only) |

   - A remap moves each use to the new number.
   - An orphan room's characters go to a fallback room: **good and neutral** characters to
     **1/297, the Bank of Godfrey** (Silvermere's bank), **evil** characters (Outlaw and
     worse, 40 evil points and up) to **1/2327, Blue Tower, Bottom Floor**. Orphan items
     and monsters are removed and listed.
   - Each moved or reset character gets a message at their next login, saying the world
     shifted around them and where they are now.
4. **The gate.** If the comb has anything to move and `--allow-remap` wasn't given, the
   install stops with a warning listing every character and object it would move. Because
   install runs at start, **the server doesn't start** until `silvermere content install
   --allow-remap` is run. `--dry-run` prints the list without changing anything.
5. **Claims.** `content_claims` is set to the sidecar's mapping.

**Recovering from a collision:** the server refuses to start; `silvermere-world build
--against <db>` keeps the names that are fine and moves the colliding ones; rebuild;
`silvermere content install --dry-run`, then `--allow-remap`.

**Ownership.** `world.rs` decides ownership by range today (anything at 9000+ is the
content's). With claims it decides by `content_claims`, which is what
[content-overlay.md](content-overlay.md#checks-and-failures) planned as `content_installs`.
The current hand-numbered content is adopted into the claims with the numbers it has, so
no character moves on the first install.

## Editing stock records

The editor changes and deletes **stock** records as well as adding new ones: rooms (exits
included), NPCs and other monsters, items, spells, shops, and their texts and messages.

**Patches.** A stock edit is a **patch**: the stock record by number, and only the fields it
changes. The builder saves one when a stock record is changed in the editor; the compiler
writes it into `world.toml` as a change record for its kind (`[[room_change]]`,
`[[monster_change]]`, `[[item_change]]`, `[[spell_change]]`, `[[shop_change]]`, ...).
Today's `[[room_change]]` (exits, shop, NPC, script) and `[[item_change]]` (abilities) grow
into these. Patches number nothing, so the claims aren't involved: the installer reads the
stock record, sets the patched fields and writes it back, at every start. A re-import
reverts stock and the next start puts the patches back, as with the rest of the overlay.

**Drift.** For every patched field the compiler also writes the **stock value the patch was
made against** (`base`). At install, a field must hold either its `base` (stock, not yet
patched) or the patched value (already patched). Anything else means stock changed
underneath the edit (a different import, a stock update), and **the install refuses: the
server doesn't start**, naming each record and field. The fix is to look at the new stock
value in the editor, keep or redo the edit, and recompile.

**Undoing an edit.** The database keeps no copy of a field's stock value once it's patched.
Removing a patch from the builder leaves the patched value in place, so undoing a stock
edit needs `silvermere import` and a restart, which writes stock back; the remaining
patches go on again at that start. The installer remembers which fields it has patched
(`content_patches`) so it can warn: "1/2140 `exits.east` was edited by the content and no
longer is; run `silvermere import` to restore stock."

**Deleting stock records.** Allowed, and handled like orphans:

- A deleted room's characters go to the fallback rooms through the comb (the bank, or the
  Blue Tower if evil), and are told at next login. A deleted item is taken out of
  inventories, floors and shop stock; a deleted monster's live copies are removed.
- Deleting changes live state, so it sits behind the same gate: without `--allow-remap` the
  install stops and lists what would move.
- References left pointing at a deleted record (an exit into a deleted room, a shop selling
  a deleted item, a monster casting a deleted spell, a script giving a deleted item) are
  warnings at compile time, listed in the report below.

**The orphan report.** Deleting records and changing stock exits can cut parts of the world
off. `silvermere-world report`, and a panel in the app, list:

- **orphaned rooms**: rooms nothing leads to any more (no exit, Map Change, teleport or
  script gets a player there), walking from the stock start rooms;
- **orphaned maps**: maps no room on another map leads into;
- **dangling references**: anything that names a deleted record.

The compiler prints the report as warnings on every compile, and the map can colour
orphaned rooms.

## The app

One native window: the 3D map in the middle, egui panels around it. Bevy 0.20 with
`bevy_egui` 0.43, as the prototype. Windows desktop first.

### The 3D map

The prototype tested the pipeline on Newhaven: rooms as cubes, exits as pipes, mouse
picking, a panel with the clicked room's data.

- **Rooms** are cubes coloured by room type (normal, shop, arena, hotel, lair, ...); dark
  rooms are darker; rooms at the edge of what's loaded are small balls. Ours and stock are
  told apart (outline or tint).
- **Exits** are pipes coloured by kind (plain, key or door, hidden, text or action,
  class/race/level, guard, ...); one-way exits are thinner. A **misfit** (rooms that
  couldn't sit where the direction says) is drawn but flagged.
- **Levels.** Up and down are real height. Isolating the selection's level fades the rest.
- **Labels.** Numbers or names over the rooms, drawn by egui at the projected positions.
- **Camera.** Orbit, pan, zoom; frame the selection (F or a double-click). Selecting a room
  in any list moves the camera to it.
- **Links between mechanisms.** A Remote Action is drawn to the exit it works on (the door
  it swaps, the hidden exit whose block it clears); a key exit to where its key comes from
  (a monster that carries it, a shop that sells it, a script that gives it).
- **Stock rooms, editable.** Stock rooms have no positions, so an area is laid out from
  its exits: breadth-first from a room, one cell per exit in its direction, the nearest
  free cell where the geometry doesn't close. The walk extends as you move; layouts are
  cached in an editor-only file and can be pinned by hand. A `/room` ID from the game or
  mudparty jumps straight to a room. Stock rooms open in the same forms as ours; a change
  saves a patch ([Editing stock records](#editing-stock-records)), fields changed from stock
  are marked, and edited stock rooms are tinted on the map. Deleting a stock room is a form
  action too.
- **Building.** Our rooms sit at their own `at` positions. **Dig** from a selected room
  (ours or stock) in a direction makes a new room in the next cell and a two-way exit; from
  a stock room that's a patch on the stock side.

### Rooms: navigation

Every exit kind gets a form with its fields by name (from `ExitParams` in
`silvermere-types`). Exits lead to rooms on the same map, except Map Change.

| Group | Kind | Fields |
|---|---|---|
| Plain | Normal | none |
| | Ticket, User Count, Delay | none (the game gives them no rules; they pass as plain) |
| | Block Guard | none (plain for players; keeps a patrolling guard monster in) |
| Locks | Door, Gate | lock, bash and pick bonus (above 0 never locks), steps it stays open, key item |
| | Key | key item, lock, pick bonus, steps it stays open |
| Requirements | Item | item carried, refusal message, message through |
| | Toll | gold crowns |
| | Class, Race | the only class (race), or the one barred |
| | Level | lowest and highest level |
| | Alignment | evil-point range |
| | Ability | ability, min and max, refusal message |
| | Timed | message (opening, closing, refusal), steps between swaps, state |
| Hidden | Hidden | search flags, blocks (1-10) still in the way, message when the last goes, message naming it |
| | Spell | none: not shown, refuses walking; passed only by typed words or following a leader |
| Words | Text | phrases (a message's three texts), message through |
| | Action | message; moves a player who can be seen |
| Remote | Remote Action | phrases, action and the far room's exit, message, item that loses a charge |
| Hazards | Trap | damage, state, message when it springs, message when a disarm fails |
| | Spell Trap | spell, state, message when disarmed, message when it springs |
| | Cast | spell before, spell after, message through, message LOOK shows |
| Maps | Map Change | map |

### Rooms: interaction

| Area | Fields |
|---|---|
| Type | Normal, Shop (shop, trainer or bank), Arena, Lair, Hotel, Colliseum, Jail, Library |
| Flags | Protected (and its message), Patrol, Owned, Cleanup, Entry spell, Gang house |
| Effects | room spell, pick-lock trap spell, light (negative is dark), death room, hang-up room |
| Contents | the NPC always here, items placed in the room |
| Monster generation | group, min and max level, max monsters, area max, controlling room, regen delay |
| Script | the room's commands ([Scripts and talk](#scripts-and-talk)) |

What the forms can set is what the game-side format can install; fields it can't yet are
shown read-only until the format grows.

### Scripts and talk

Scripts are **raw text** to start, with tools; a rule builder (trigger, conditions,
effects, output) comes later.

- **Highlighting and completion** for the 42 keywords, grouped as conditions (Class, Race,
  GoodAligned, EvilAligned, MinLevel, MaxLevel, CheckItem, FailItem, CheckAbility,
  FailAbility, TestAbility, CheckSkill, TestSkill, CheckSpell, RoomItem, FailRoomItem,
  NeedMonster, NoMonsters, Monsters, Random, TestTournament, Flag), effects (GiveItem,
  TakeItem, HideItem, ClearItem, GiveAbility, AddAbility, RemoveAbility, AddExp, AddEvil,
  GiveCoins, Price, Cast, LearnSpell, Teleport, Summon, RemoteAction, AddDelay) and output
  (Text, RoomText, Message).
- **Names for references.** `takeitem hounds-leash 0:text bonded-hound`, with the text
  written inline or by name; the compiler writes the numbers.
- **Which line wins.** Lines are tried in order and the first that holds runs, so a script
  leans on fall-through lines (`use leash:class 17:text ...` after the line that bonds).
  The editor answers "for a player like this (class, abilities, items), typing this, which
  line runs?" and flags lines that can never run.
- **Linting** with the shared parser: every clause parses, every line has a `:`, every
  reference exists.
- **Talk.** An NPC's ASK text (`KEYWORD:block` lines, the greeting linked) shown as a tree:
  keyword, answer, linked block.

### The other editors

Each editor creates new records and patches or deletes stock ones, in the same forms
(fields changed from stock are marked). Monster logic is **only what the stock data
supports**: no new behaviour, so no engine changes. Order: monsters, items, spells, then
the rest.

| Editor | What it edits | Worth building in |
|---|---|---|
| Monsters | stats, abilities, 5 attack slots (cumulative chance, kind, accuracy, damage, energy, hit/miss/deflect messages, spell on a hit), 5 spells after rounds, items carried with chances, coins, generation type, group and role, followers, follow chance, max alive, regen hours, spells on creation and death, talk and random-event texts | expected damage per round against an AC; where it spawns, on the map |
| Items | type, abilities, classes and races allowed, spells negated, weapon and armour stats, charges, container coins, hit and miss verbs, text after the description, limited copies | sections per type (weapon, armour, container, key, scroll); where it comes from (shops, drops, scripts, rooms) |
| Spells | about 30 fields, with level scaling (`maxval += maxinc * level / maxlvls`, the same for min and duration) | the computed values at levels 1-50 |
| The rest | messages, text blocks, shops (stock, training class and levels) | forms |

Each one needs the **game-side format extended first** (a `[[spell]]` record and its
`[[spell_change]]`, more fields on `[[monster]]`, `[[item]]` and `[[room]]` and on their
change records, deletes), in silvermere's `silvermere-content`, before
the editor can ship it: the compiler can only write what the game installs.

## Checks

| Check | Where | On failure |
|---|---|---|
| The builder files parse; every name is known; values in range; descriptions within their lines | the app, live; the compiler; MMUD_Builder CI | the compile fails, naming the record |
| Script lines parse and every reference exists | the app (shared parser); the compiler | the compile fails |
| A script line that can never run | the app | a warning |
| Gotchas: `max_monsters` set, dark rooms meant to be dark, texts within 1999 bytes a line | the app; the compiler | the compile fails, or a warning |
| A Hidden exit's blocks match the Remote Actions that clear them; every key is obtainable; a door matches from both sides; a Text exit's phrases don't collide with a script pattern | the app; the compiler | a warning |
| A number held by something the content doesn't own | install | the server doesn't start |
| A remap or orphan without `--allow-remap` | install | the server doesn't start; the list is printed |
| The compiled file's version stamp differs from the installer's | install | a warning |
| A patched stock field holds neither its `base` nor the patched value (drift) | install | the server doesn't start, naming the record and field |
| A field the content patched before and no longer does | install | a warning: run `silvermere import` to restore stock |
| A stock delete without `--allow-remap` | install | the server doesn't start; the list is printed |
| Orphaned rooms and maps; references to deleted records | the compiler (`silvermere-world report`); the app | a warning |

## Fitting it into silvermere

| Piece | Where | Change to existing files |
|---|---|---|
| `silvermere-world` (compiler, library and CLI) | a new crate, main workspace | the workspace list |
| The script parser | a new crate, out of `silvermere-engine/src/scripts` | the engine uses it instead of its own copy |
| `content_claims` and `content_patches` tables | new migrations, sqlite and postgres | none |
| Change records for every kind, with `base` values; deletes | `silvermere-content`'s `model.rs` and `world.rs` | today's `[[room_change]]` and `[[item_change]]` grow into them |
| Claims, collision check, comb, login messages | `silvermere-content`'s install | `world.rs` decides ownership by claims, not by range |
| `--allow-remap`, `--dry-run`, `content claims export` | `bin/silvermere`'s `content` subcommand | the CLI list and `docs/cli.md` |
| The sidecar | `content/world.claims.toml` | `Content::from_files` skips it |
| Extending the format, per editor | `silvermere-content`'s `model.rs` and `world.rs` | new records and fields, one editor at a time |

The importer, the stock files and the engine's game logic are untouched.

## Implementation order

1. **Viewer** (MMUD_Builder): the prototype made into the app's skeleton. Stock browsing
   with the layout cache and pinning, search, `/room` jumps, loading from a DB copy or
   `/v1/backup`. Useful at once for exploring the world.
2. **Builder format and compiler** (silvermere's `silvermere-world`): allocation from
   `claims.lock` or the instance. Proved by converting today's `world.toml` into builder
   files once and compiling it back to the same records.
3. **Claims and the comb** (silvermere): the table, the collision check, the comb, the
   fallback rooms, login messages, `--allow-remap`, `--dry-run`, `claims export`. Can run
   alongside 4.
4. **Room editing** (MMUD_Builder): our rooms and stock rooms on the map, the room and exit
   forms for both (stock edits as patches, with the drift check and deletes), dig, saving
   builder files, Compile, the orphan report.
5. **Checks**: the shared script parser split out, script linting and which-line-wins, the
   cross-room checks, MMUD_Builder CI.
6. **The other editors**, each after its game-side format, new and stock records together:
   monsters, items, spells, then messages, shops and the rest.
7. **Later**: live re-install and `/v1/reload-world` without a restart; the rule builder.

## Decisions

1. **The editor edits the builder format, not the database or `world.toml`.** `world.toml`
   becomes a compiled file, in exactly the format silvermere reads today.
2. **MMUD_Builder** is its own public repo: the app and the builder source files.
3. **The compiler and the script parser are small crates in silvermere**, shared with the
   game; MMUD_Builder pins them by commit.
4. **A native app in Rust: Bevy and `bevy_egui`**, with a 3D map. Not a TUI, not paged 2D
   levels.
5. **Numbers are allocated at compile time**, from the instance's used numbers (the
   instance directly or `claims.lock`). Editor positions are separate from game numbers.
6. **Our content installs last.** Anything installed after it allocates around it.
7. **Claims are kept in the database** (`content_claims`); moving a claimed number runs the
   comb. Orphans go to the Bank of Godfrey (1/297) if good or neutral, the Blue Tower
   (1/2327) if evil, and are told at next login.
8. **A remap needs `--allow-remap`**, or the install stops with a warning and the server
   doesn't start.
9. **Stock rooms are on the 3D map**, laid out from their exits, and editable.
10. **Scripts are raw text with tools** first; the rule builder later.
11. **Monster logic is only what the stock data supports.**
12. **Stock records can be changed and deleted**: rooms, NPCs and monsters, items, spells,
    shops, texts and messages. Changes are patches of only the changed fields; deletes go
    through the comb.
13. **Editors after rooms: monsters, items, spells**, then the rest; each does new and stock
    records together.
14. **Drift refuses the start.** A patched field whose stock value isn't the one the patch
    was made against stops the install until the edit is redone.
15. **Undoing a stock edit needs a re-import** (`silvermere import` and a restart); the
    installer warns about fields it patched before and no longer does.
16. **Orphaned rooms and maps are reported**, with references to deleted records, by the
    compiler and the app.

## Open questions

1. **Secrets in a public repo.** Quest solutions and scripts in MMUD_Builder are public.
   Keep it that way, or keep area files in a private repo the public app opens?
2. **Getting the compiled file into silvermere.** Copied by hand, a script, or a PR opened
   by MMUD_Builder's CI?
3. **The fork ranges.** Allocation prefers them (9000+ and so on) so the content is easy to
   spot in the database and in `/room`; is falling back above a full range enough, or
   should it fail?
4. **The format's first steps.** Which room fields (flags, generation, room spell, light)
   join the game-side format first, with room editing?

## Brainstorm

- A **walkthrough mode** on the map: pick a class, level and items, and see which exits
  open, which script lines fire and which doors a lever swaps.
- **Diffs on the map**: what a compile changes against the last install, as coloured
  rooms and pipes, before installing.
- **mudparty** sending the party's current room to the editor, so the map follows a live
  game.
- Exporting a stock area into builder files as a starting point for a copy.
- Keeping each field's stock value in the database the first time it's patched, so an edit
  could be undone without a re-import (set aside in favour of the re-import, Decision 15).
- The rule builder writing the same raw lines, so the two can be mixed in one script.
