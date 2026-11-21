# War Mage and Spellstrike

Status: **design settled, not implemented.** Every open question is decided. The class
record and help topic depend on the content overlay, designed in [content-overlay.md](content-overlay.md). Fork-only: this departs from
MajorMUD 1.11p, so it never goes upstream to jnfisher/silvermere.

- [Summary](#summary)
- [Lore](#lore)
- [The class](#the-class)
- [Level titles](#level-titles)
- [Spellstrike](#spellstrike)
- [Warlocks later](#warlocks-later)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Brainstorm](#brainstorm)

## Summary

The **War Mage** is a fighter first and a caster second: fighting 4 (the same as a Warrior), Magic Mage-1, chainmail,
any one- or two-handed weapon, no shield. Its feature, **Spellstrike**, lets it cast an
attack spell *and* swing its weapon in the same combat round: the cast takes the place of
the first blow, and a failed cast counts as a missed blow. Spellstrike is built as a class
ability so warlocks can be given it later by data alone.

## Lore

The stock help topic for warlocks (`WCCMMHLP.MSG`, topic 26) calls them the front-line
fighters of magical conflict, and says "The King's Battle-Mage corps is comprised almost
exclusively of Warlocks." Warlocks favour light armour (no heavier than chain) for casting
speed, and keep one hand free to direct ranged spells, so they use one-handed weapons only.

The War Mage is the other half of that corps: the soldiers who don't keep a hand free.
They hold a blade or a greatsword two-handed and drive the spell *through* the weapon at
close range instead of throwing it. That's the in-world reason they can't carry a shield
and can use two-handed weapons, and the reason their magic is weaker than a warlock's
(Mage-1 against Mage-2).

Help text to write (our own words, not the original's): a `War Mages` topic in the same
box layout as the stock class topics, with the line
`HP/Level: 5-9  Magic: Mage-1  Combat: Good  Weapons: Any  Armour: Chain` and
`Abilities: Spellstrike.` The hidden ability is not listed.

## The class

The stock classes it sits between, from `WCCCLASS.VIR` and the help topics. "Fighting" is
the 1-5 scale the class help uses (Poor, Fair, Average, Good, Excellent); the record's
`combat` value is fighting + 2.

| Class | HP/level | Exp | Magic | Fighting (`combat`) | Weapons | Armour |
|---|---|---|---|---|---|---|
| Witchunter | 6 + 1d4 | 130% | none | 5 (7) | All | Scalemail |
| Warrior | 6 + 1d4 | 100% | none | 4 (6) | All | Platemail |
| Paladin | 5 + 1d4 | 220% | Priest-1 | 4 (6) | All | Platemail |
| Ranger | 5 + 1d4 | 250% | Druid-1 | 4 (6) | All | Rigid Leather |
| Warlock | 4 + 1d4 | 185% | Mage-2 | 3 (5) | Any 1-H | Chainmail |
| Mage | 3 + 1d3 | 140% | Mage-3 | 1 (3) | Limited (staff) | Robes |
| **War Mage** | **5 + 1d4** | **220%** | **Mage-1** | **4 (6)** | **All** | **Chainmail** |

The class record:

| Field | Value | Note |
|---|---|---|
| `name` | `War Mage` | 8 of the 30 bytes the record allows |
| `magery` / `magelvl` | `Mage` / `1` | Spellcasting rises like other Mage-1 hybrids |
| `weapon` | `All` (8) | one- and two-handed, blunt and sharp |
| `armour` | `Chainmail` (7) | |
| `combat` | `6` | fighting 4, the same as Warrior and Paladin |
| `basehp` / `hplevel` | `5` / `4` | 5-9 HP per level |
| `titles` | text block `3019` | the War Mage's own ranks, [below](#level-titles) |
| `exp` | `120` | 220% experience |
| abilities | `Spellstrike` (30), `SpellstrikeSuccessOff` (10, hidden), `SpellstrikeFailOff` (40, hidden), `NoOffHand`, plus `Bash` (31), which every stock class has | new codes, below; each holds one value |

**No shield.** Stock has no class rule that refuses shields as such. The only one there is
"a class limited to one-handed weapons may not wear an off-hand item"
(`crates/silvermere-engine/src/items/can_use.rs`), and the War Mage uses every weapon. A new
class ability, `NoOffHand`, refuses any item worn `OffHand`, with the same refusal message
as armour the class can't wear. `Class Item Inclusion` on an item still overrides it, as it
overrides the other class checks.

### Level titles

Sixteen ranks, in the same level bands as every stock class:

| Levels | Title | Levels | Title |
|---|---|---|---|
| 1 | Apprentice Recruit | 35-39 | Major Artilerist |
| 2-4 | Apprentice Soldat | 40-44 | Arcane Aspirant |
| 5-9 | Arcane Brigadier | 45-49 | Sub-Lieutenant |
| 10-14 | Runic Brigadier | 50-54 | Lieutenant |
| 15-19 | Runic Corporal | 55-59 | Arcane Captain |
| 20-24 | Arcane Sergeant | 60-64 | Commandant |
| 25-29 | Chief Sergeant | 65-69 | Colonel Arcane |
| 30-34 | Arcane Adjutant | 70-110 | Arch-Colonel |

Arch-Colonel runs to level 110 like the stock last ranks; above that the game's built-in
fallback titles apply, as they do for every class. The block is a new text block,
3019. Stock's titles are 3001-3015, one per class; 3016-3018 hold title lists for three
classes stock never shipped (Samurai, Artificer, Pirate), so the War Mage doesn't take
3016.

## Spellstrike

### How a player uses it

There's no new command. A character whose class has `Spellstrike` who is **wielding a weapon** and
**CASTs an attack spell in combat** (by hand or through auto-combat) makes a spellstrike
round instead of a casting round. Unarmed, or with a spell that isn't an attack spell, the
cast is stock. MegaMud's existing `CAST` scripting keeps working.

"Attack spell" means any spell the character can cast that deals damage
(`DamageNoArmour`, `DamageArmour` or `StealHp`) to a single target. (`Damage` is not
one: it's the ability that adds to a fighter's blows.) Area (room-target) attack spells are
not spellstrikes: cast while armed, they make a stock casting round.

### The round

Stock, for reference: a round gives 1000 energy (`TICK_ENERGY`); melee blows cost
`compute_energy_used(level, combat, agility, weapon speed)` each, at least 200, at most 6
blows. A casting round makes no blows and repeats the spell while its energy lasts.

A spellstrike round:

1. **The cast goes first**, in the place of the first blow, at the auto-combat target.
2. The cast chance is stock (`spellcast + diff` against the roll).
3. **The cast costs one blow.** The energy charged is what one blow with the wielded
   weapon costs, with every stock adjustment (load, strength below the weapon's
   requirement, being slowed) and the 200 floor, then discounted as in the table below.
   The spell's own `energy` field isn't used in a spellstrike. That keeps stock's
   full-round attack spells (Magic Missile's `energy` is 1000) usable, and it means a
   faster weapon makes a cheaper spellstrike.
4. **On success:** the spell lands as normal (damage, resist, messages). Mana is charged in
   full, as stock charges it.
5. **On failure:** stock's fail line (`You attempt to cast X, but fail.`), which reads as
   the missed blow. Mana is charged as stock charges a failure (half).
6. **The rest of the pool buys melee blows** as a normal round would, so at most 5 blows
   after the cast (the cast counts as one of the 6). Leftover energy carries into the next
   round as stock's does.
7. **No after-cast extra.** Stock makes a blow cost 10% more after its attacker has cast
   in the same round. A spellstrike's blows don't pay it, and neither does the cast.
   Outside a spellstrike the stock rule is unchanged.
8. **Out of mana.** If the caster can't pay the spell's mana when a spellstrike round
   starts, the stock line `You do not have enough mana to cast that spell` is shown once,
   and the fight carries on as plain melee: that round is a normal melee round, not lost.
   It stays melee for the rest of the fight, even if mana comes back. Spellstrike starts
   again only when the player types CAST again. (Stock casters short of mana in auto-combat
   lose the round instead and keep trying.)
9. One cast per round. The spell doesn't repeat the way a stock casting round does.

Energy charged for the cast, as a share of one blow's cost. The discount is taken after
the 200 floor, so a discounted cast can cost under 200.

| | Success | Failure |
|---|---|---|
| `Spellstrike` alone (warlock, later) | 100% | 70% (30% off) |
| War Mage (`Spellstrike` + `SpellstrikeSuccessOff` + `SpellstrikeFailOff`) | 90% (10% off) | 30% (70% off: 30 + 40) |

### Worked example

How a blow's cost is worked out, and how weapon speed plays into it, is in
[Round energy](#round-energy). The figures here are a War Mage (`combat` 6) with Agility 50,
starting a round at full energy with no carry. Discounted costs round down.

**Level 10, speed-1200 weapon (the stock quarterstaff), carrying nothing:** one blow
costs 256.

| Round | Energy spent | Blows | Carried |
|---|---|---|---|
| Plain melee | 3 × 256 | 3 | 232 |
| Spellstrike, cast succeeds | cast 230 + 3 × 256 | cast + 3 | 2 |
| Spellstrike, cast fails | cast 76 + 3 × 256 | (miss) + 3 | 156 |

**The same, half loaded:** one blow costs 342.

| Round | Energy spent | Blows | Carried |
|---|---|---|---|
| Plain melee | 2 × 342 | 2 | 316 |
| Spellstrike, cast succeeds | cast 307 + 2 × 342 | cast + 2 | 9 |
| Spellstrike, cast fails | cast 102 + 2 × 342 | (miss) + 2 | 214 |

**Any blow at the 200 floor** (most weapons from about level 25):

| Round | Energy spent | Blows | Carried |
|---|---|---|---|
| Plain melee | 5 × 200 | 5 | 0 |
| Spellstrike, cast succeeds | cast 180 + 4 × 200 | cast + 4 | 20 |
| Spellstrike, cast fails | cast 60 + 4 × 200 | (miss) + 4 | 140 |

What this means in play:

- **Early on, a success costs about one blow a round.** Sometimes it only uses up leftover
  energy and keeps the blow count (the level 10 quarterstaff above), but over several
  rounds it comes to roughly one blow per cast (see the 3-round comparison below).
- **At the floor, a success trades exactly one blow for the spell**, as first intended.
- **A failure is cheap at every level.** The miss costs less than a blow, and the energy
  saved carries forward. To make a failure cost a full blow, set both failure discounts
  to 0; they're values in the class's abilities, not code.

### Cast chance

Stock: the chance is the caster's Spellcasting plus the spell's difficulty, at most 98%,
against a roll of 0-99. A Mage-class caster's Spellcasting is
`2 × level + (3 × Int + Wil) / 6 + 5 × magery level`, plus Spellcasting from items and
quests. The War Mage's magery level is 1, so it's 5 points behind a warlock (Mage-2) and 10
behind a mage (Mage-3) at the same stats and level.

The War Mage's best single-target attack spell at each level (stock Mage-1 spells), and
its cast chance with Int 50 / Wil 50 (Spellcasting base 33) and Int 80 / Wil 60 (base 50),
before any items:

| Level | Best spell | Difficulty | Damage | Chance (Int 50, Wil 50) | Chance (Int 80, Wil 60) |
|---|---|---|---|---|---|
| 1 | magic missile | +15 | 4-12 | 55% | 72% |
| 5 | frost jet | +5 | 8-19 | 53% | 70% |
| 10 | lightning bolt | 0 | 16-32 | 58% | 75% |
| 15 | sunbolt | -14 | 30-55 | 54% | 71% |
| 20 | eldritch bolt | -30 | 40-88 | 48% | 65% |
| 25 | eldritch bolt | -30 | 40-108 | 58% | 75% |
| 30 | eldritch bolt | -30 | 40-120 | 68% | 85% |
| 35 | eldritch bolt | -30 | 40-120 | 78% | 95% |

Mage-1 has no single-target attack spell above level 20 (eldritch bolt), and eldritch
bolt stops growing at level 28. The War Mage spells ([Decision 8](#decisions)) are where
that gap gets filled.

### Speed bonus to the cast

Stock gives a **fast blow** (one that would cost under 200 energy before the floor) a
bonus to its critical chance: `200 − cost + (Agility − 50) / 10`, at most 20, halved at
33-66% load, none above 66% or when too weak for the weapon. In a spellstrike that bonus
**goes to the cast first**: it's added to the cast chance up to the 98% cap, and the
blows get only what's left over as critical chance. A War Mage at 90% with a +20 bonus
casts at 98% and swings with +12 crit; one at 60% casts at 80% and swings with no bonus. It's worked out from the same blow the cast costs
([The round](#the-round), step 3). Because only
fast weapons earn it, it gives one-handers a real edge in casting while two-handers keep
their damage.

The bonus a War Mage earns (Agility 50, carrying nothing):

| Level | Dagger (900) | Shortsword (1000) | Longsword (1350) | Broadsword (1500) | Bastard sword (1700) | Greatsword (2200) |
|---|---|---|---|---|---|---|
| 1, 5 | 0 | 0 | 0 | 0 | 0 | 0 |
| 10 | +8 | 0 | 0 | 0 | 0 | 0 |
| 15 | +20 | +20 | 0 | 0 | 0 | 0 |
| 20 | +20 | +20 | +17 | 0 | 0 | 0 |
| 25 | +20 | +20 | +20 | +20 | +5 | 0 |
| 30 | +20 | +20 | +20 | +20 | +20 | 0 |
| 35 | +20 | +20 | +20 | +20 | +20 | +7 |

So at level 20 with Int 50 / Wil 50, eldritch bolt lands 65% of the time through a
longsword and 48% through a greatsword; at level 25, 78% through a broadsword and 58%
through a greatsword. See [open question 11](#decisions).

### Three rounds with a broadsword

Melee swings over 3 rounds (no carry at the start), Agility 50, carrying nothing, Strength
at least 50 (the broadsword's requirement). Broadsword: speed 1500, 3-13 damage. The cost
of one blow is in brackets.

| Level | Warlock (`combat` 5) | War Mage (6) | Warrior (6) | Witchunter (7) |
|---|---|---|---|---|
| 5 | 6 (481) | 6 (450) | 6 (450) | 7 (421) |
| 10 | 8 (354) | 9 (321) | 9 (321) | 10 (293) |
| 15 | 10 (281) | 12 (249) | 12 (249) | 13 (225) |
| 20 | 12 (232) | 14 (204) | 14 (204) | 15 (200) |

The same 3 rounds as spellstrikes (each round a cast plus swings; the table counts swings,
and every round also casts the spell):

| Level | War Mage, every cast succeeds | War Mage, every cast fails | Warlock with Spellstrike (later), succeeds | Warlock, fails |
|---|---|---|---|---|
| 5 | 3 | 5 | 3 | 4 |
| 10 | 6 | 8 | 5 | 6 |
| 15 | 9 | 11 | 7 | 8 |
| 20 | 12 | 13 | 9 | 10 |

The War Mage swings exactly as a Warrior does, because they share `combat` 6; the
difference is armour, shield and the spell. Each successful spellstrike costs about one
swing a round, so over 3 rounds the War Mage trades 3 swings for 3 spells. Half loaded,
every class loses about a quarter of its swings (Warrior 5, 7, 9, 11).

### Round energy

How stock works, for reference ([`silvermere-rules`' `combat/energy.rs` and
`combat/fighter.rs`](../crates/silvermere-rules/src/combat/)):

- **A round gives 1000 energy** (one round every 5 seconds in auto-combat). Blows are made
  while the energy left can pay for one, at most 6, and what's left carries into the
  next round.
- **A blow's base cost** is `speed × 1000 / ((level × combat + 45) × (agility + 150) / 6)`,
  plus the weapon's own extra energy (0 on every stock weapon). Higher speed is slower;
  level × combat is where the class's fighting shows.
- **Then, in order:** × 0.75 (carrying nothing) to × 1.25 (fully loaded); +1.5% per point of
  strength below the weapon's requirement; speed × 1.5 when slowed; +10% after casting in
  the same round (not in a spellstrike); × 2 for a bash, the whole round for a smash;
  then at least 200 and at most 1000.
- **A blow under 200 before the floor** is a fast blow and raises the critical chance.

Stock weapon speeds (`WCCITEMS`, after the update file):

| Type | Weapons | Fastest | Typical | Slowest |
|---|---|---|---|---|
| 1-H blunt | 85 | 1000 | 1600 | 3200 |
| 1-H sharp | 148 | 700 | 1650 | 3000 |
| 2-H blunt | 64 | 1000 | 2000 | 5000 |
| 2-H sharp | 82 | 100 | 2060 | 4600 |

Examples: dagger 900, longsword 1350, bastard sword 1700, battle axe 1725, greatsword
2200 (6-21 damage, Strength 75).

A War Mage's blow cost, carrying nothing, Agility 50 (blows per round in brackets):

| Level | Dagger | Longsword | Battle axe | Greatsword |
|---|---|---|---|---|
| 1 | 396 (2-3) | 595 (1-2) | 760 (1) | 970 (1) |
| 10 | 200 (5) | 288 (3-4) | 369 (2-3) | 471 (2) |
| 25 | 200 (5) | 200 (5) | 200 (5) | 253 (3-4) |
| 50 | 200 (5) | 200 (5) | 200 (5) | 200 (5) |

Weapon speed matters early and fades: by about level 25 nearly every weapon is at the
floor, and from then on a two-hander's slowness costs nothing while its damage still
counts.

### Messages

A spellstrike round opens with a lead-in line, then the stock cast lines (the hit, or the
fail line), then the stock melee lines:

| Who | Line |
|---|---|
| The caster | `You channel magic missile through your longsword!` |
| The room | `Gorin channels magic missile through his longsword!` |
| The target (a player) | `Gorin channels magic missile through his longsword at you!` |

The lead-in is shown on every spellstrike, before the roll, so a failure reads as "channel,
then fail" and stays the missed blow. It uses the spell's name and the weapon's name as
the game shows them elsewhere, and the possessive pronoun stock uses in combat lines. The
lines are new fork messages in `silvermere-text`, coloured like the stock cast lines.

MegaMud reads combat text line by line. A line it doesn't know is passed over, so the
lead-in shouldn't break its combat tracking. Check that with MegaMud once it's built.

## Warlocks later

Everything is keyed to class abilities, so warlocks get Spellstrike by adding the
`Spellstrike` ability to class 11's record in the content overlay. No code change. They get
the base discounts only (not the two hidden abilities), they keep their one-handed weapons, and
they can still wear a shield (no `NoOffHand`). Their wider Mage-2 spell list carries over
automatically, because Spellstrike takes any attack spell the caster knows.

One stock behaviour changes for them: an armed warlock casting an attack spell in combat
always makes spellstrike rounds instead of casting rounds. There's no option to turn it
off. A warlock who wants a stock casting round (the spell repeated while energy lasts)
unwields the weapon first, or casts an area spell.

**Never for stealth classes.** Spellstrike is for the battle-mage line. No class with
stealth ever gets it: not Bards, Gypsies, Missionaries or Rangers, even though they cast,
and not Mystics, Ninjas or Thieves. (These are the stock classes with the Class Stealth
ability.)

## Fitting it into silvermere

The fork stays mergeable with upstream, so the change keeps to new files plus a few small,
marked hooks.

| Piece | Where |
|---|---|
| New ability codes `Spellstrike`, `SpellstrikeSuccessOff`, `SpellstrikeFailOff`, `NoOffHand` | the `Ability` enum in `crates/silvermere-types/src/enums.rs`, in a fork-only range well above stock's last (187), so an upstream addition can't collide. Each ability's one value is a discount percentage: `Spellstrike` the base failure discount, the other two the War Mage's extra success and failure discounts. They're tuned in the content files, not code. |
| Energy arithmetic (charges, discounts, blows left) | new `crates/silvermere-rules/src/combat/spellstrike.rs`, pure and proptest-able |
| The round | new `crates/silvermere-engine/src/combat/spellstrike.rs`, one hook in the `casting` branch of the round in `combat/autocombat.rs` |
| Hidden abilities | not listed by the help or the class report |
| No shield | one check in `items/can_use.rs` |
| The class record and help topic | the fork's content overlay, installed into the database at every start ([content-overlay.md](content-overlay.md)) |

When no class has `Spellstrike`, every hook falls through to stock, so the faithfulness
tests and the differential harness still pass on stock data.

## Decisions

1. ~~Energy with full-round spells.~~ **Decided:** the cast costs one blow's energy, with
   the discounts applied to it ([The round](#the-round)). The other options, set aside: a
   fixed share of the spell's energy; a pool of 1000 plus one blow; cheaper War Mage-only
   spells.
2. ~~"Fighting 4".~~ **Decided:** Warrior's fighting, record `combat` 6.
3. ~~HP and experience.~~ **Decided:** 5-9 HP per level and 220% experience, the
   Paladin's numbers.
4. ~~Area spells.~~ **Decided:** single-target attack spells only. Area spells cast the
   stock way.
5. ~~A lead-in line.~~ **Decided:** yes, for the caster, the room and a player target
   ([Messages](#messages)).
6. ~~Warlocks' choice.~~ **Decided:** armed warlocks always spellstrike, with no `SET`
   option.
7. ~~Races.~~ **Decided:** every race may be a War Mage.
8. **War Mage spells.** Decided that War Mages get new spells of their own, in a separate
   update after the class. Those spells will be shared with warlocks, since the two
   classes are close cousins in a way mages are not, so they get their own design covering
   both. The class ships first with the stock Mage-1 list. Spellstrike already takes any
   single-target attack spell the class can cast, so new spells need no change here.
9. **Level titles.** The War Mage has its own titles, the ranks of the Battle-Mage corps.
   It doesn't borrow the warlock's, which already use "Warrior Mage", "Battlemage" and
   "Warmage" as ranks. Same title for either sex. See [Level titles](#level-titles).
10. **After-cast extra.** Decided: a spellstrike is exempt from stock's 10% extra on blows
    after a cast ([The round](#the-round), step 7).
11. **Speed bonus to the cast.** Decided: in a spellstrike round the fast-blow bonus goes
    to the cast first. Only the part that would take the cast chance past 98% goes to the
    round's blows as critical chance. Outside a spellstrike, blows keep it as stock.
12. **Out of mana.** Decided: a spellstrike that can't pay its mana switches the fight to
    plain melee for the rest of the fight, with no lost round, and doesn't switch back on
    its own ([The round](#the-round), step 8).
13. **No Spellstrike for stealth classes.** Decided: no class with Class Stealth ever gets
    Spellstrike: Bards, Gypsies, Missionaries, Rangers, Mystics, Ninjas and Thieves.

## Brainstorm

Ideas that came up, not decided:

- Weapon-charge spells ("searing edge", "frost brand") as candidates for the War Mage
  spell list (see Decision 8).
- Spellstrike damage could scale with the weapon (a share of the weapon's damage added to
  the spell), so the class rewards its two-handers.
