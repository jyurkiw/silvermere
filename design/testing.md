# Testing the fork

Status: **design settled, not built.** Fork-only.

- [Summary](#summary)
- [What silvermere already has](#what-silvermere-already-has)
- [The class roster](#the-class-roster)
- [Arena scenario tests](#arena-scenario-tests)
- [Running against a local server](#running-against-a-local-server)
- [Kits](#kits)
- [The balance sim](#the-balance-sim)
- [Casting budgets](#casting-budgets)
- [The War Mage runs](#the-war-mage-runs)
- [Reading the results](#reading-the-results)
- [Decisions](#decisions)
- [Open questions](#open-questions)

## Summary

Three kinds of test, all covering **every class**:

1. **Arena scenario tests** in `cargo test`: pre-built characters fight the practice dummy
   on a real server, through a real telnet client. They check that the server keeps
   running and the text is right.
2. **The same scenarios against a local server** you run from `E:/Repos/silvermere`, with
   logging, when you want to watch.
3. **A balance sim**: every class against a spread of stock monsters at several levels,
   1000 rounds a matchup, on the game's own combat code. It answers "does each class earn
   its experience percentage", with the Warrior at 100% as the baseline.

Unit tests still cover the rules on their own; these test the game as it runs.

## What silvermere already has

- **Live-server tests** (`bin/silvermere/tests`, [docs/testing-live-sessions.md](../docs/testing-live-sessions.md)):
  the real server started in the test on the imported stock world, driven by the
  testkit's `TelnetClient`. Helpers create characters, edit their records, place them in
  rooms and read back what was saved. `start_manual` stops the real-time clock and `tick`
  sends combat ticks by hand, so a 5-second round takes no time.
- **Text classification**: the repo's pattern registry sorts every line of output and
  flags lines it doesn't know.
- **The arena**: room 1/206, *Arena Practice Room*, has a *practice dummy* (monster 200:
  9999 HP, no armour, no experience). Room 1/2519 has a *hardened practice dummy* (monster
  603: AC 75, DR 75).
- **For a running server**: the Python telnet bridge, `tools/smoke/smoke.py`, and the
  scripted-session format of `tools/differential`.

## The class roster

One file, `tests/roster.toml` (name to settle), with a standard test character per class:
the 15 stock classes and the War Mage, and any class added later. Each entry gives:

- race and stats (one standard spread per class, so classes are compared fairly);
- the test levels (1, 5, 10, 15, 20, 25, 30, 35);
- **a kit per level band**, chosen by the [kit rules](#kits);
- the class's attacks: `attack` for fighters; backstab then attack for the stealth classes;
  martial arts for the Mystic; spells by magic type; Spellstrike for the War Mage.

Both the scenario tests and the sim read the roster, so a new class is one new entry.

## Arena scenario tests

Each class, at each test level, in room 1/206 against the practice dummy:

1. the character is created from the roster and placed in the room;
2. it attacks the dummy with its class attack (cast, backstab, spellstrike as it has);
3. a few rounds are run by sending ticks;
4. checks:
   - **the server keeps running**: no panic in the log, no disconnect;
   - **every line is known**: the pattern registry recognises all output;
   - **new text is exact**: for fork text (the Spellstrike lead-in, its order against the
     cast and blow lines), the exact lines;
   - **the numbers**: energy and mana after the round match the design (from the record
     the test reads back).

Scenarios are data, not Rust: a file of characters and steps (send, expect, ticks), so the
same file runs in `cargo test` and against a running server. They run on every build.

## Running against a local server

- Serve from `E:/Repos/silvermere` with the log on (`silvermere serve`, data in `data/`).
- A fork command loads the roster's characters into the database (server stopped), in the
  same style as the content overlay. Name to settle (`silvermere dev seed`).
- The scenario runner connects to `localhost` and plays the same scenario files, printing
  each step and PASS or FAIL.

## Kits

Every test character gets a standard kit for its level, chosen by rules from the
stock data rather than by hand, so every class is treated the same way.

**Where the gear comes from: shops, opening up by level band.**

| Band | Levels | Shops |
|---|---|---|
| Town | 1-9 | Silvermere's ordinary shops: Helfgrim's Blades, Thuluk's Axes, Colin's Maces, Skali's Fine Armour, Sentara's Clothing and Leatherworks, the Robes and Shield shops, and Newhaven's |
| Travelled | 10-19 | the above, plus the Dwarven, Elven, Rhudaur and Arlysia shops |
| Veteran | 20+ | the above, plus Jorah's Tower and the Lost City shops |

Only gear sold in shops: no drops, no limited items, nothing from the sysop or gang
shops. Price is ignored: gold was always far easier to come by than experience, so what
holds a character back is where it can safely go, which is what the bands stand for.

**What a character may take:** what the game would let it use. That covers class and
armour type, class and race lists, the item's minimum level, and the weapon's Strength
requirement against the character's Strength. War Mages take no shield.

**The weapon:** the one with the most expected damage per round for that character at
that level. That's average damage times blows per round, worked out the game's way, so a
fast weapon for a weak character can beat a heavy one. For the War Mage, the [speed
bonus](war-mage.md#speed-bonus-to-the-cast) makes this a trade-off, so it runs with both
its best one-hander and its best two-hander.

**The armour:** for each worn slot, the item with the most armour class plus damage
reduction. The total load is kept under a third of what the character can carry, so the
kit never costs the character its speed bonus or blows.

**The stats:** Human for every class, with the class's creation minimums and its points
spent in a fixed order per class. Fighters favour Strength, Agility and Health; casters
favour Intellect or Willpower. The orders are in the roster, so they can be changed.

The sim writes out the kit it picked for every class and level, so you can check and
override any of them in the roster.

## The balance sim

A fork tool that loads the real stock world and runs fights straight on the engine, with
a seeded random generator. It uses the game's own combat and spell code, not a copy of
the formulas, so the numbers are the game's.

For each class, at each test level, against each monster in the spread:

- **Both sides get huge HP and are healed to full every round**, so nobody dies and the
  numbers settle.
- **1000 rounds**, in **5-round fights**: every 5 rounds the fight restarts, so openers
  (backstab, a first-round cast) happen once a fight, and buffs and effects reset.
- **Casting follows the class's casting budget** ([below](#casting-budgets)), not real
  mana.
- **Every option is tried** where the class has a choice (plain attack, bash, smash, each
  attack spell), and the best is reported with the runners-up.
- **Recorded every round**: damage out, damage in, hits, misses, crits, casts and cast
  failures, and the spell used.

**The monster spread**: for each level band, a handful of stock monsters around that level
that players actually fight, picked from the database by level and where they're placed.
The list is a file you can edit.

## Casting budgets

Mana isn't tracked. Each class gets a number of casts per 5-round fight, by how much it
depends on magic. Casts come first in the fight (round 1, then round 2); the other rounds
are melee.

| Tier | Classes | Casts per 5-round fight |
|---|---|---|
| Pure caster | Mage, Priest, Druid | every round (5) |
| Caster-2 | Warlock, Gypsy, Cleric, Missionary, Bard | 1 or 2, by the spell ladder (below) |
| Caster-1 | Paladin, Ranger | always 1 |
| War Mage | | its three runs ([below](#the-war-mage-runs)) |
| No casting | Warrior, Witchunter, Ninja, Thief | 0 |
| Kai | Mystic | 0: martial arts only |

**Caster-2: 1 or 2 casts.** A class's *spell ladder* is its best single-target attack spell
at each level, from the stock spells of its magic type. Each spell is held from the level it
arrives until the level the next one replaces it. In the **first half** of that span the
caster gets **1** cast per fight; from the **halfway point** on, **2**, because by then it
has the mana for its current spell. The next upgrade then costs more and fails more, and
the count drops back to 1. A ladder's **last** spell, which nothing replaces, gets 1 cast
for the first 5 levels after it arrives and 2 from then on.

Example, a Warlock: frost jet arrives at level 4 and lightning bolt replaces it at 8. The
halfway point is 6, so a Warlock casts frost jet once a fight at levels 4-5 and twice at
6-7, then lightning bolt once at 8.

The sim works the ladders out from the stock spells for each magic type, so they follow the
data.

## The War Mage runs

The War Mage is a caster-1 class, but Spellstrike is what's being tested, so it runs three
times:

| Run | Casting |
|---|---|
| **Opener** | its best spell as a spellstrike in rounds 1 and 2, then plain melee for rounds 3-5 |
| **Conserve** | a spellstrike every round with a cheap spell: the one **two tiers below** its best (magic missile, frost jet or lightning bolt) |
| **Die, die, die** | a spellstrike every round with its best attack spell |

The War Mage's damage ladder, from its Mage-1 spells: magic missile (level 1), frost jet
(4), lightning bolt (8), sunbolt (13), eldritch bolt (20). For *Conserve*: at eldritch bolt
it casts lightning bolt; at sunbolt, frost jet; at lightning bolt, magic missile. Below
that (levels 1-7) it casts magic missile.

When other classes get Spellstrike (warlocks first), they get the same three runs. Stealth
classes never get Spellstrike ([war-mage.md](war-mage.md#warlocks-later)).

## Reading the results

Everything is against the **Warrior at 100%**, at the same level:

- **Performance**: the class's damage out, and damage in, as a share of the Warrior's.
- **Price**: its experience percentage over 100 (a Warlock 1.85, a War Mage 2.2).
- **Worth**: how its performance compares with its price. A class paying 185% should do
  about 1.85 times as well as a Warrior to earn it.

Also derived per matchup: rounds to kill the monster (its real HP over damage out), and HP
lost per kill (damage in over that time).

**Equal effort.** Levelling is slow, and with 9 lives before a reroll, experience cost
mostly decides *how high a character gets*, not how fast. So the report also compares
classes at the same experience spent: for the experience a Warrior needs to reach level
L, the level each other class reaches (from the game's own experience table), and how that
class at that level does against the Warrior at L. A Warlock that's 1.6 times a Warrior's
damage at equal level, but a few levels behind at equal experience, shows up here.

**Survival counts.** Lives are scarce, so damage in is reported beside damage out, not
folded into one number.

What the sim measures is **solo fighting**. Party value (healing, cures, blessings) and
utility (stealth, picklocks, traps) aren't scored; the report names them for each class so
a low solo score isn't read as "overpriced". Party tests can come later.

## Decisions

1. Every class is tested, through one roster.
2. The Warrior at 100% is the baseline.
3. Balance: huge HP, full heal every round, 1000 rounds, every class against a spread of
   stock monsters at several levels, best options at every level.
4. Casting budgets per 5-round fight, by tier, with the caster-2 1-or-2 rule at the
   halfway point of the spell ladder.
5. Caster-1 classes cast once a fight; the War Mage runs Opener, Conserve and Die, die,
   die. Other classes get the same three runs when they get Spellstrike.
6. Fights restart every 5 rounds, which also sets how often openers happen.
7. Mystics fight with martial arts only.
8. Kits come from stock shops only, by the rules in [Kits](#kits).
9. A ladder's last spell switches to 2 casts 5 levels after it arrives.
10. Bards cast their attack songs on their ladder.
11. Stealth classes never get Spellstrike, so they never get the three Spellstrike runs.
12. Draining spells (vampiric touch, deathtouch) are not on the spell ladders.

## Open questions

All decided:

1. ~~Fight length.~~ **Decided:** fights restart every 5 rounds.
2. ~~Mystic.~~ **Decided:** martial arts only, no casting budget.
3. ~~Conserve below lightning bolt.~~ **Decided:** at levels 1-7 Conserve casts magic missile every round.
4. ~~The kits.~~ **Decided:** the rules in [Kits](#kits), shop gear only.
5. ~~The last rung of a ladder.~~ **Decided:** 1 cast for the first 5 levels after a ladder's last spell arrives, then 2.
6. ~~Bards.~~ **Decided:** the Bard's ladder uses its attack songs.
7. ~~Draining spells.~~ **Decided:** vampiric touch and deathtouch stay off the ladders.
   They cost too much mana to cast routinely; players keep them for emergencies.
