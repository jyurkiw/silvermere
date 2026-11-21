# War Mage: implementation plan

Status: **plan, not started.** Builds what [war-mage.md](war-mage.md) and
[content-overlay.md](content-overlay.md) decided. War Mage spells are a later update
([war-mage.md, Decision 8](war-mage.md#decisions)).

- [Where the code lives](#where-the-code-lives)
- [Order of work](#order-of-work)
- [Step 1: the content overlay](#step-1-the-content-overlay)
- [Step 2: the new abilities and No shield](#step-2-the-new-abilities-and-no-shield)
- [Step 3: the War Mage content](#step-3-the-war-mage-content)
- [Step 4: Spellstrike rules](#step-4-spellstrike-rules)
- [Step 5: the Spellstrike round](#step-5-the-spellstrike-round)
- [Step 6: play-testing](#step-6-play-testing)
- [Working rules](#working-rules)
- [Risks](#risks)

## Where the code lives

All in the fork, in silvermere's own crates. A sibling project could hold the content and
its install tool, but Spellstrike and the no-shield rule change compiled engine code, so
the fork is needed either way. Keeping it all here is simpler to build and test, and the
mergeability rule below keeps the changes to existing files small.

## Order of work

Six steps, each one branch and one PR on the fork, each leaving the server working. The
overlay comes first because nothing else can be seen in play without it. Steps 4 and 5
don't need step 1 to be written, so they could go in parallel, but each needs a War Mage in
the world to test end to end.

| Step | What | Depends on | Size |
|---|---|---|---|
| 1 | Content overlay: classes and HELP topics, installed at start | none | largest |
| 2 | The four new abilities, the no-shield rule | none | small |
| 3 | `content/` for the War Mage: class, titles, HELP topic | 1, 2 | small, data only |
| 4 | Spellstrike energy and cast-chance rules, pure functions | 2 | small |
| 5 | The Spellstrike round in combat, the lead-in lines | 2, 4 | medium |
| 6 | Play-testing with telnet and MegaMud, tuning | 3, 5 | open |

## Step 1: the content overlay

Design: [content-overlay.md](content-overlay.md).

| Piece | Files |
|---|---|
| New crate `silvermere-content`, from `crates/_template/` | `crates/silvermere-content/`; add it to the workspace members and `[workspace.dependencies]` in the root `Cargo.toml` |
| TOML model and reading: `[[class]]`, `[[class.change]]`, `[class.titles]`, `[[class_topic]]`, `[[topic]]` | `src/model.rs`, `src/read.rs` (serde + the `toml` crate already pinned for config) |
| Names to codes (abilities, weapon and armour types, magery) | `src/names.rs`, using `silvermere-types`' names |
| Checks: everything known and in range; changes name records that exist | `src/check.rs` |
| Turning the model into records: `Class`, the titles `TextBlock` (one line per level, 1-110) | `src/build.rs` |
| Built into the binary | `crates/silvermere-content/build.rs`, like `crates/silvermere-web/build.rs` embeds `docs/` |
| `content_installs` table: kind, number, content hash | new migration in `migrations/sqlite/` and `migrations/postgres/`, repository in `crates/silvermere-db/src/repo/`; regenerate `docs/schema.md` |
| Install: write new records, apply changes, refuse numbers owned by other content, warn on records no longer listed | `src/install.rs` |
| Hook at start: install after the world-import check, before the world loads | `bin/silvermere/src/commands/serve.rs` (a few lines) |
| `silvermere content install` and `silvermere content check` | `bin/silvermere/src/cli.rs`, `bin/silvermere/src/commands/content.rs`; document in `docs/cli.md` (the docs-in-sync test checks it) |
| HELP topics: add or replace topics when the help text loads; draw `[[class_topic]]` in the stock class-topic box from the class record | `crates/silvermere-text/src/package.rs` (a few lines calling in), the drawing in `silvermere-content` |

The crate depends on `-types`, `-db` and `-text` only, so it sits beside the importer in
the dependency rule.

**Tests:** parsing and checks on small fixture files; an install into a SQLite test
database that runs twice and gives the same rows; a collision test (class 16 already
present and not ours); a re-import test (import, install, re-import with
`--content-only`, install: the change to a stock class is back); a HELP test that finds an
overlay topic. Use a test-only class in fixtures, not the War Mage, so step 1 stands alone.

**Done when:** a server started on a fresh stock database with a fixture class in
`content/` shows it at character creation and in `HELP CLASSES`, and stock behaviour is
otherwise unchanged (the full test suite passes).

## Step 2: the new abilities and No shield

| Piece | Files |
|---|---|
| `Spellstrike`, `SpellstrikeSuccessOff`, `SpellstrikeFailOff`, `NoOffHand` in the `Ability` enum, at codes in a fork range well above stock's last (187) | `crates/silvermere-types/src/enums.rs`, and their names in `names.rs` |
| Hidden abilities: kept out of any place a player sees their class's abilities | find where class abilities are listed to players (the class report and stat displays under `crates/silvermere-engine/src/display/`) |
| `NoOffHand`: refuse items worn `OffHand`, with the armour refusal message; `Class Item Inclusion` still overrides | `crates/silvermere-engine/src/items/can_use.rs` (a few lines) |

**Tests:** an item test for the no-shield rule, including the Class Item Inclusion
override; a test that a class without `NoOffHand` is unchanged.

## Step 3: the War Mage content

`content/classes.toml` and `content/help.toml`, exactly as in
[content-overlay.md, The files](content-overlay.md#the-files): class 16 with its stats and
abilities, titles block 3019, and the `War Mages` HELP topic in our own words (the
lore paragraph from [war-mage.md, Lore](war-mage.md#lore)).

**Tests:** `content check` passes as a test; a character created as a War Mage gets the
right HP range, Spellcasting, title at levels 1, 25 and 70, and is refused a shield.

## Step 4: Spellstrike rules

Pure functions in a new `crates/silvermere-rules/src/combat/spellstrike.rs`:

- the cast's energy: one blow's cost (all stock adjustments, the 200 floor, no after-cast
  extra), then the success or failure discount from the ability values, rounded down;
- the speed bonus: the fast-blow bonus of that blow, added to the cast chance up to 98%,
  and the remainder as critical chance for the round's blows;
- the cast chance: Spellcasting plus difficulty plus the speed bonus, at most 98%.

**Tests:** proptests over levels, stats and weapon speeds (costs never negative, never
above a blow's cost, the cap holds, the bonus split always adds up), and unit tests that
reproduce the worked example and tables in [war-mage.md](war-mage.md#worked-example).

## Step 5: the Spellstrike round

| Piece | Files |
|---|---|
| The round: when the caster's class has `Spellstrike`, a weapon is wielded and the spell is a single-target attack spell, cast first at the blow's discounted cost, then blows from what's left; one cast a round | new `crates/silvermere-engine/src/combat/spellstrike.rs`; one hook in the `casting` branch of the round in `crates/silvermere-engine/src/combat/autocombat.rs` |
| By-hand CAST while armed starts spellstrike rounds | the same hook through `crate::spells::cast` |
| No after-cast extra for a spellstrike's blows | the `CANCAST` rule in `crates/silvermere-rules/src/combat/fighter.rs` skipped for a spellstrike |
| Mana as stock: full on success, half on failure | reuse the cast's charging in `crates/silvermere-engine/src/spells/cast.rs` |
| Lead-in lines for the caster, the room and a player target | new messages in `crates/silvermere-text/messages/catalogue.toml` and `Msg` variants in `crates/silvermere-text/src/msg.rs` |

**Tests:** engine tests with a seeded RNG for a spellstrike that succeeds and one that
fails (energy, blows, carry, mana, message order); area spells and unarmed casts stay
stock; a class without `Spellstrike` is untouched; an `insta` snapshot of a spellstrike
round's text.

## Step 6: play-testing

- Telnet and MegaMud against a local server: make a War Mage, fight at a few levels,
  check MegaMud still tracks combat with the lead-in line.
- Compare swings and cast rates against the tables in [war-mage.md](war-mage.md).
- Tune the three ability values in `content/classes.toml` if needed; that's a content
  change, not code.

## Working rules

- **Nothing is committed or pushed without your go-ahead**, step by step.
- Branches off the fork's `main`, one per step; PRs go to `jyurkiw/silvermere`, never
  upstream.
- The repo's checks on every step: `cargo fmt`, `cargo clippy -D warnings`, the full test
  suite on SQLite (PostgreSQL where available), `cargo deny check`, the docs checks.
- Stock behaviour must not change: every new hook falls through when no class has the new
  abilities, so upstream's tests keep passing and upstream merges stay easy.
- Comments describe behaviour, never the original's internals (the repo's wording guard).

## Risks

- **Ability codes.** If upstream ever adds abilities, the fork range keeps them apart, but
  any stored character data with fork codes means a database can't go back to upstream's
  build. That's expected for a fork; worth knowing before a real server runs it.
- **The differential harness** compares against the original under MBBSEmu. It runs on
  stock data, so it should be unaffected, but it can't check anything fork-only.
- **MegaMud and the lead-in line.** Untested until step 6. If it confuses MegaMud, the
  fallback is a line shaped like an existing stock combat line.
- **Hidden abilities.** Step 2 has to find every place class abilities reach a player;
  missing one shows the War Mage's discounts.
