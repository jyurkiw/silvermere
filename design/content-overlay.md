# Content overlay

Status: **built** on branch `feature/war-mage` (classes, titles and HELP topics). Fork-only.

- [Summary](#summary)
- [Why it's needed](#why-its-needed)
- [How it works](#how-it-works)
- [The files](#the-files)
- [Help topics](#help-topics)
- [Checks and failures](#checks-and-failures)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

The fork's own content (the War Mage first) is written as TOML files in `content/` in
the repo. The files are built into the `silvermere` program, and the server **installs them
into the database every time it starts**, before it serves. Nobody has to remember to
install anything: a first start, a re-import from the stock files or an upgrade all end up
with the content in place on the next start. The stock files and the importer are never
changed.

## Why it's needed

The world comes from the stock 1.11p files. `silvermere import` reads them once, applies the
stock update file `WCCUPDAT.DAT`, and writes the database; after that the server reads only
the database. An import writes each record by its number, adding it or overwriting it,
and leaves records the stock files don't mention alone. So:

| Our content | After a re-import over the same database | After a new database |
|---|---|---|
| A new record (the War Mage, class 16) | kept | gone |
| A change to a stock record (the warlock with Spellstrike) | reverted to stock | gone |

Whatever we add has to be put back after any import. Doing it at every start makes that
automatic.

## How it works

```
first start / silvermere import:   stock files ──► WCCUPDAT.DAT ──► database
every start (silvermere serve):     database ──► install content/ ──► load the world ──► serve
```

1. `serve` opens and migrates the database as it does today.
2. **Install.** For each record in the built-in `content/`:
   - **a new record** (`[[class]]`) is written whole, by its number;
   - **a change** (`[[class_change]]`) reads the stock record, changes only the fields it
     names, and writes it back.

   Writing the same content twice gives the same database, so installing at every start is
   safe. It's a handful of records, so it doesn't slow the start down.
3. Ownership is checked by name: a new class's number held by a class of another name (a
   sysop's own class 16 from a world editor) is refused, and so is its titles block when
   the class wasn't already there. No table records the install. (As built: the design
   first had a `content_installs` table; the name check does the same job without a
   schema change.)
4. The world loads from the database as it does today. From here on nothing knows the
   content came from the overlay: the War Mage is a class like any other, and appears in
   the class list at character creation without code changes, because that list is built
   from the classes in the database.

*Not built yet:* `silvermere content install` would run the same step without serving (for checking a change
before a restart), and `silvermere content check` checks the files without touching a
database. The server holds a single lock on the database, so `install` needs it stopped, as
`import` does.

The admin API's world reload re-reads the database only. Content changes need a new build
and a restart anyway, so that's enough.

## The files

TOML, one file per kind of record, in `content/` at the repo root. Field names follow the
game's help and class report where those exist, not the raw record names, and the
installer turns them into record values.

`content/classes.toml`:

```toml
# The War Mage (design/war-mage.md).
[[class]]
number    = 16
name      = "War Mage"
hp        = "5-9"          # HP per level: basehp 5, hplevel 4
exp       = 220            # percent, as the help shows it (record value 120)
magic     = "Mage-1"
fighting  = 4              # 1-5 as the help shows it (record combat 6)
weapons   = "All"
armour    = "Chainmail"
abilities = [
  { name = "Bash" },
  { name = "Spellstrike",           value = 30 },   # base failure discount, percent
  { name = "SpellstrikeSuccessOff", value = 10 },
  { name = "SpellstrikeFailOff",    value = 40 },
  { name = "NoOffHand" },
]

# Level titles: one entry per band, written out to one line per level (1-110) in text
# block 3019. "Lord|Lady" style pairs work as in stock.
[class.titles]
block = 3019
ranks = [
  { from = 1,  title = "Apprentice Recruit" },
  { from = 2,  title = "Apprentice Soldat" },
  { from = 5,  title = "Arcane Brigadier" },
  { from = 10, title = "Runic Brigadier" },
  { from = 15, title = "Runic Corporal" },
  { from = 20, title = "Arcane Sergeant" },
  { from = 25, title = "Chief Sergeant" },
  { from = 30, title = "Arcane Adjutant" },
  { from = 35, title = "Major Artilerist" },
  { from = 40, title = "Arcane Aspirant" },
  { from = 45, title = "Sub-Lieutenant" },
  { from = 50, title = "Lieutenant" },
  { from = 55, title = "Arcane Captain" },
  { from = 60, title = "Commandant" },
  { from = 65, title = "Colonel Arcane" },
  { from = 70, title = "Arch-Colonel" },   # to level 110
]

# Later: warlocks get the base Spellstrike.
# [[class_change]]
# name = "Warlock"
# add_abilities = [{ name = "Spellstrike", value = 30 }]
```

- **New records name their number.** The class number is what a player types at
  character creation (`[16] War Mage`), so the fork uses the next free number after
  stock's 15, not a far-off range. The install refuses a number that some other content is
  already using ([Checks](#checks-and-failures)).
- **Changes name the stock record**, by name or by number, and list only what they
  change. A change to a record that isn't there is an error.
- **Names, not codes, everywhere:** abilities, weapon and armour types, magery. The
  installer looks them up in the names silvermere already has (`silvermere-types`'
  `names.rs`), plus the fork's new abilities.
- **Each ability holds one value**, as the class record's ability slots do. Spellstrike's
  two discounts are two abilities.

## Help topics

HELP text isn't in the database. The first import unpacks `WCCMMHLP.MSG` into
`data/assets/`, and the server reads it at every start. Overlay help topics don't touch that
file: they're read from the built-in `content/help.toml` at the same point and added to the
topic list before the end marker. A topic with the same words as a stock one replaces it.

```toml
# A class topic, drawn in the same box layout as the stock class topics, from the class's
# own record, so the stat line can't drift from the numbers.
[[class_topic]]
class = "War Mage"
text = '''
War Mages are the soldiers of the King's Battle-Mage corps who would rather not keep a
hand free. ...
'''
abilities = "Spellstrike."     # the hidden ability isn't listed

# Any other topic: the text as printed.
[[topic]]
words = "Spellstrike"
text = '''
...
'''
```

`HELP CLASSES` lists classes from the database already, so the War Mage appears there with
no help entry.

## Checks and failures

| Check | When | On failure |
|---|---|---|
| The TOML parses, every name is known, every value is in range | build (a test) and `content check` | the build's tests fail |
| A change names a record that exists | install | the server doesn't start, and says which record |
| A new record's number (a class, or a titles text block) isn't used by content the overlay doesn't own (a sysop's own class 16 from a world editor) | install, by the class's name | the server doesn't start, and says which number |
| The content hash matches the last install | install | not a failure: a different hash is logged as "content updated" |

Refusing to start is deliberate. A server that starts without the War Mage would load War
Mage characters with no class.

**Removing content.** Deleting a `[[class]]` from the files doesn't delete it from the
database, because characters may still be in that class. The install logs a warning naming
records the overlay owns but no longer lists.

## Fitting it into silvermere

| Piece | Where | Change to existing files |
|---|---|---|
| The TOML files | new `content/` | none |
| Built into the program | a build script, as `silvermere-web` builds in `docs/` | none |
| Reading, checking, installing | a new crate, `silvermere-content`, depending on `-types` and `-db` like the importer | add it to the workspace lists |
| Install at start | one call in `serve` after the migrate step | a few lines |
| `content install` / `content check` | not built yet: the install runs at start and the checks run as tests | |
| Overlay help topics | one call where the help topics are loaded | a few lines in `silvermere-text`'s package loading |

The importer, the stock files and the engine are untouched. The engine changes the War
Mage needs are in [war-mage.md](war-mage.md#fitting-it-into-silvermere).

## Decisions

1. TOML files.
2. In the fork repo, `content/`, built into the program.
3. Install and forget: installed into the database at every server start, never into the
   stock files.
4. The first version covers **classes** (new, and changes to stock ones, including a
   class's level titles) and **help topics**, which is what the War Mage needs. Titles
   are written as a text block, but only as part of a class. Spells come with the spells
   design. Races, general text blocks and the rest come when something needs them.

## Open questions

All decided:

1. ~~Scope.~~ **Decided:** classes and help topics ([Decision 4](#decisions)).
2. ~~Two-value abilities.~~ **Decided:** one value per ability. The War Mage has
   `Spellstrike` (30), `SpellstrikeSuccessOff` (10) and `SpellstrikeFailOff` (40).
3. ~~Level titles.~~ **Decided:** the War Mage has its own titles, written with the class
   (`[class.titles]`) as text block 3019.

## Brainstorm

- `silvermere content export class 11` printing a stock record as overlay TOML, as a
  starting point for a change.
- The same overlay could later carry new areas (rooms, monsters, items) as the content
  work grows.
