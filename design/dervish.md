# The Dervish: two blades

Status: **implemented** (`feature/dervish`). Fork-only.

As built: the off-hand weapon sits in a worn slot (any worn slot, since a weapon's own
`worn` place is "nowhere"), which is how its bonuses count as a worn item's. `DualWield` is
fork ability 1017. The two hands take turns only in auto-combat against monsters; a Dervish
fighting a player uses its main hand alone for now.

- [Summary](#summary)
- [Where it sits](#where-it-sits)
- [The class](#the-class)
- [OFFHAND](#offhand)
- [Two hands, two rounds](#two-hands-two-rounds)
- [How many blows](#how-many-blows)
- [Messages](#messages)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

The Dervish fights with a one-handed weapon in each hand. Each hand gets its own round of
blows, so a Dervish swings far more often than the stock limit of blows per round allows
anyone else. It always swings at least twice a round, once with each hand.

- Fighting 5 (`combat` 7, the Witchunter's), leather armour, no magic, no stealth.
- One-handed melee weapons only: no shields, no two-handed weapons, no ranged weapons.
- A new command, **OFFHAND**, puts a second weapon in the off-hand.

## Where it sits

| Class | Fighting | Armour | Weapons | Shield | HP | Exp |
|---|---|---|---|---|---|---|
| Warrior | 4 | plate | any | yes | 6-10 | 100% |
| Witchunter | 5 | chainmail | any | yes | | |
| Ninja | 3 | ninja | any | yes | 4-8 | 185% |
| **Dervish** | **5** | **leather** | **1-H melee, two at once** | **no** | **5-9** | **200%** |

## The class

| Field | Value | Note |
|---|---|---|
| class number | 20 | after the Assassin (19) |
| HP per level | 5-9 | lighter armour than a Warrior, so a little less |
| experience | 200% | raised from 130% after play-testing: the extra blows earn it |
| magic | none | |
| fighting | 5 | `combat` 7 |
| weapons | `AnyOneHand` | stock already bars a one-handed class from off-hand (shield) items |
| armour | leather | `StuddedRigidLeather` |
| abilities | **DualWield** *(fork)* | value: the off-hand's accuracy penalty |
| titles | text block 3023 | proposed below |

Ranged weapons: the Warden's `Ranged` marker already marks them, so a class without
`RangedWeapons` that also has `DualWield` is refused any item marked `Ranged`, in either hand.

**Lore (proposed).** The Dervishes come out of the desert dancing. Their fighting is a
spinning dance taught in desert lodges, two blades turning around the dancer. They wear
nothing heavier than leather, because a dancer in mail can't turn. They don't trust shields
("the second blade is the shield") or bows. In a fight, a Dervish's foe faces a whirl of
steel, and the blows come faster than it can count.

**Titles (proposed),** in the stock level bands:

| Levels | Title | Levels | Title |
|---|---|---|---|
| 1 | Novice | 35-39 | Bladewind |
| 2-4 | Dancer | 40-44 | Storm Dancer |
| 5-9 | Spinner | 45-49 | Whirlwind |
| 10-14 | Twinblade | 50-54 | Sandstorm |
| 15-19 | Blade Dancer | 55-59 | Sword Saint |
| 20-24 | Whirler | 60-64 | Dance Master |
| 25-29 | Dervish Adept | 65-69 | Master of the Lodge |
| 30-34 | Steel Dancer | 70+ | Dervish |

## OFFHAND

- **`OFFHAND <item>`** (new): puts a weapon from inventory into the off-hand. It does what
  WIELD does: the same checks (the class can use it, it isn't cursed into place, and so on),
  messages shaped like WIELD's ("You are now wielding the shortsword in your off-hand."), and
  it breaks off auto-combat, as changing equipment does.
- **Who:** only a class with `DualWield`. Anyone else gets the stock
  unknown-command reply, so the command stays hidden.
- **What:** a one-handed melee weapon the class can wield. Two-handed and `Ranged` weapons are
  refused.
- **Where it's kept:** the stock off-hand slot (`worn[WORN_OFFHAND]`), where a shield
  normally sits. A Dervish can't hold a shield, so the slot is free. The slot is saved with
  the character, so this needs no new table.
- **A main weapon first:** OFFHAND needs a weapon already wielded in the main hand.
  Otherwise: "You must wield a weapon in your main hand first."
- **When the main weapon comes off** (REMOVE, a drop, a theft, a break): the off-hand weapon
  moves into the main hand, and the off-hand is left empty. "You shift the shortsword to your
  main hand." The Dervish then fights with one weapon: a stock round.
- **Taking it off:** REMOVE works as it does for any worn item. INVENTORY and the equipment
  list show it as "(Off-Hand)".
- **Swapping hands:** `OFFHAND` with the main-hand weapon's name moves it to the off-hand,
  if the off-hand is empty.

## Two hands, two rounds

Each hand gets a stock round of its own. The main hand is exactly the stock round.

- **Energy:** each hand has its own energy pool, 1000 a round, with its own carry-over. A
  blow costs `compute_energy_used(level, combat, agility, that weapon's speed)` plus the
  load factor, floored at 200, as now. So a fast off-hand dagger swings more often than a
  slow main-hand mace.
- **The order:** the hands take turns: main, off, main, off..., until both have run out of
  energy. A hand that runs out first drops out and the other keeps going.
- **The limit: 8 blows a round, both hands together.** Since the hands take turns, two hands
  with energy to spare get 4 each. When one hand runs out, the other goes on until the total
  reaches 8, though never past the stock 6 for one hand (`MAX_BLOWS`).
- **The minimum:** each hand swings at least once a round, even when its weapon costs more
  than a round's energy. That is the "always two swings". A slow weapon's extra cost isn't
  carried over as a debt: the hand's carry-over just goes to 0.
- **The off-hand blow:** a stock blow made with the off-hand weapon's damage, speed,
  accuracy, verbs and abilities. It takes the class's `DualWield` penalty to accuracy:
  −10, at every level, with full damage.
- **Critical hits:** the fast-blow critical bonus (`200 − cost + (agi−50)/10`, at most 20)
  works out per hand from that hand's own blow cost.
- **What ends both hands:** a kill, the Dervish fleeing or being stunned, or the fight
  breaking off. As stock, a kill ends the round.
- **Special moves:** BASH, SMASH and BACKSTAB don't apply (the class has none of them). An
  explicit `ATTACK` is the first blow of the main hand's round.

## How many blows

At Agility 50, with a speed-1000 weapon in each hand. A stock fighter's figure is one
hand's.

| Level | Cost per blow | Witchunter (blows a round) | Dervish (both hands) |
|---|---|---|---|
| 1 | 577 | 1-2 | 2-4 |
| 5 | 375 | 2-3 | 4-6 |
| 10 | 260 | 3-4 | 6-8 |
| 15+ | 200 (the floor) | 5 | 8 (the cap) |

The figures come from `cost = speed × 1000 / ((level × 7 + 45) × 200 × 1500 / 9000)`.
The minimum of one blow per hand matters for slow weapons. A speed-2000 weapon costs 1154 a
blow at level 1, which would otherwise be one blow every other round.

A Dervish lands twice the Witchunter's blows until the cap, and 8 to its 5 once both are
at the floor, less the off-hand penalty, with lighter armour and no shield. Its experience
was first set at 130%, on par with the Witchunter, and raised to 200% after play-testing:
the extra blows are worth more than the Witchunter's heavier armour and shield. The balance sim in [testing.md](testing.md) should
check the penalty and the experience.

## Messages

MegaMud reads combat lines byte for byte (learned with the Warden's companion). So an
off-hand blow is told with exactly the stock hit, miss and critical lines, using the
off-hand weapon's own verbs: "You slice the kobold thief for 4 damage!" There is no
"(off-hand)" tag and no new line shape, so MegaMud counts every blow. The room sees the
stock room lines.

## Fitting it into silvermere

| Piece | Where |
|---|---|
| The class, its titles and HELP | content (`content/classes.toml`, `content/help.toml`), like the War Mage and the Warden |
| Fork ability `DualWield` (1017) | `silvermere-types` `fork` |
| OFFHAND | a new command (an input hook, like `PET`), reusing WIELD's checks and the equip path into `worn[OffHand]` |
| Moving the off-hand weapon over when the main one comes off | one hook wherever `weapon` is cleared (remove, drop, theft, breakage), in `items/equip.rs` |
| Refusing two-handed and ranged off-hand weapons | `items/can_use.rs`, next to the War Mage's `NoOffHand` |
| The second hand's round | auto-combat (`combat/autocombat.rs`): after the stock round's setup, the two hands' blows take turns, each through the stock blow resolution with its weapon's fighter side |
| The off-hand energy pool | a per-session map in `World`, like `pet_energy` |
| The minimum blow per hand | one condition in the round loop |
| The off-hand's worn bonuses | nothing new: stock already adds the bonuses of whatever sits in `worn[OffHand]`, as it does for a shield (to be confirmed when it's built) |

The off-hand round reuses the stock blow (`resolve`), kill (`check_kill_monster`) and
experience paths, so kills, experience and death lines stay stock.

## Decisions

1. Fighting 5 (`combat` 7), leather, no magic, no stealth.
2. One-handed melee weapons only: no shields, no two-handed weapons, no ranged weapons.
3. A new OFFHAND command puts the second weapon in the off-hand.
4. Each hand gets its own round of blows, beyond the stock blow limit. At least two blows a
   round, one per hand.
5. Off-hand blows take −10 accuracy, at full damage, the same at every level.
6. At most 8 blows a round for both hands together, taken in turns (4 each when both have
   the energy), and never more than 6 for one hand.
7. The off-hand weapon counts as a worn item: all its bonuses apply and stack with the main
   weapon's. Its on-hit abilities apply to its own blows.
8. OFFHAND needs a main weapon wielded. When the main weapon comes off, the off-hand weapon
   moves into the main hand.
9. 5-9 HP a level and 200% experience (first 130%, raised after play-testing).
10. All races may be Dervishes.

## Open questions

1. ~~HP and experience.~~ **Decided:** 5-9 HP a level, 200% experience (first 130%).
2. ~~The off-hand penalty.~~ **Decided:** −10 accuracy, full damage, the same at every
   level. It's the `DualWield` ability's value, so the balance sim can tune it.
3. ~~The blow limit.~~ **Decided:** 8 a round for both hands together, and never more than
   the stock 6 for one hand.
4. ~~The off-hand's item abilities.~~ **Decided:** all of them count. The off-hand weapon is
   a worn item like any other, so its AC, stat and accuracy bonuses add to the character's
   and stack with the main weapon's (two +5 accuracy swords give +10). Its on-hit abilities
   (Hit Magical, crits, poison) apply to its own blows.
5. ~~No main weapon.~~ **Decided:** OFFHAND needs a main weapon first, and if the main
   weapon comes off, the off-hand weapon moves into the main hand.
6. **Monsters:** monsters with two weapons are out of scope (stock monsters have one
   `weapon`).
7. **Class number and titles:** 20, block 3023.
8. ~~Races.~~ **Decided:** all races, as the War Mage.

## Brainstorm

- A Dervish whirl: one round that strikes everyone fighting it (a later ability).
- Paired weapons: a matched pair (twin scimitars) gives a bonus when worn together.
- Trainers: the Dervish lodge, in the room renovations.
