# The Avenger and the Assassin: stealth casters

Status: **draft, in design.** Nothing here is implemented. Fork-only.

- [Summary](#summary)
- [Where they sit](#where-they-sit)
- [The Avenger](#the-avenger)
- [The Assassin](#the-assassin)
- [Shared rules](#shared-rules)
- [Spell backstab (the Assassin)](#spell-backstab-the-assassin)
- [Vanish (the Avenger)](#vanish-the-avenger)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

Two fighter-leaning stealth casters. Unlike the Gypsy and the Missionary (fighting 2,
magic level 2), they fight first and cast second: fighting 3, magic level 1.

- **The Avenger** (Priest-1, blunt weapons): the hidden hand of the temples. Its signature
  is **vanish**: dropping back into hiding in the middle of a fight, to strike again from
  the shadows.
- **The Assassin** (Mage-1, sharp weapons): an arcane killer. Its signature is **spell
  backstab**: opening from hiding with an attack spell that lands like a backstab.

Both are stealth classes, so neither ever gets Spellstrike
([war-mage.md, Decision 13](war-mage.md#decisions)).

## Where they sit

| Class | Magic | Fighting | Armour | Weapons | HP | Exp |
|---|---|---|---|---|---|---|
| Thief | none | 2 | leather | 1-H | 4-7 | 80% |
| Ninja | none | 3 | ninja | any | 4-8 | 185% |
| Bard | Bardic | 3 | leather | 1-H | 4-7 | 210% |
| Gypsy | Mage-2 | 2 | leather | 1-H | 4-7 | 220% |
| Missionary | Priest-2 | 2 | leather | 1-H | 4-7 | 220% |
| **Avenger** | **Priest-1** | **3** | **leather** | **blunt** | **4-8** | **200%** |
| **Assassin** | **Mage-1** | **3** | **leather** | **sharp** | **4-8** | **200%** |

## The Avenger

| Field | Value | Note |
|---|---|---|
| class number | 18 | after the Warden |
| HP per level | 4-8 | `basehp` 4, `hplevel` 4 |
| experience | 200% | `exp` 100 |
| magic | Priest-1 | |
| fighting | 3 | `combat` 5 |
| weapons | any blunt | `AnyBlunt` |
| armour | leather | `StuddedRigidLeather`, the stealth classes' limit |
| abilities | stealth, backstab, **blunt backstab**, **vanish** | see [Shared rules](#shared-rules) and [Vanish](#vanish-the-avenger) |
| titles | text block 3021 | below |

**Lore (proposed).** When a temple names someone an enemy of the faith and no court will
touch them, it sends an Avenger. They are priests who learned to walk unseen: they carry
no holy symbol openly and wear no vestments, only dark leather and a mace. Good temples
send them after the cruel and the powerful; evil temples send them after everyone else.
An Avenger fights well enough face to face, but its art is to strike, vanish, and strike
again before its quarry knows where the blows are coming from. Good Avengers call down holy
force, and evil ones unholy force.

**Titles (proposed),** in the stock level bands:

| Levels | Title | Levels | Title |
|---|---|---|---|
| 1 | Acolyte | 35-39 | Templar Shade |
| 2-4 | Initiate | 40-44 | Vindicator |
| 5-9 | Watcher | 45-49 | Judicator |
| 10-14 | Penitent | 50-54 | Hand of Wrath |
| 15-19 | Seeker | 55-59 | Shadow Templar |
| 20-24 | Pursuer | 60-64 | Grand Inquisitor |
| 25-29 | Inquisitor | 65-69 | Avatar of Vengeance |
| 30-34 | Zealot | 70+ | Avenger |

**Its spells,** the stock Priest-1 single-target attack line (priest spells are open to every
priest level): harm (1), spiritual hammer (4), agony (8), holy force *or* unholy force by
alignment (13), godstrike (20).

## The Assassin

| Field | Value | Note |
|---|---|---|
| class number | 19 | |
| HP per level | 4-8 | |
| experience | 200% | |
| magic | Mage-1 | |
| fighting | 3 | `combat` 5 |
| weapons | any sharp | `AnySharp` |
| armour | leather | `StuddedRigidLeather` |
| abilities | stealth, backstab, **spell backstab** | see [Spell backstab](#spell-backstab-the-assassin) |
| titles | text block 3022 | below |

**Lore (proposed).** The Assassins learned their magic the way they learned their knives:
from someone who expected to be paid for it. A Mage would call their spellcraft crude, and
it is: a handful of killing words and nothing else. But spoken from the dark, at someone
who never knew they were there, a crude spell is enough. An Assassin's first strike is the
one that matters, and it is as likely to be a bolt of force as a blade.

**Titles (proposed):**

| Levels | Title | Levels | Title |
|---|---|---|---|
| 1 | Initiate | 35-39 | Shadowcaster |
| 2-4 | Footpad | 40-44 | Deathwhisper |
| 5-9 | Cutthroat | 45-49 | Executioner |
| 10-14 | Knifehand | 50-54 | Black Hand |
| 15-19 | Hexknife | 55-59 | Master of Knives |
| 20-24 | Nightblade | 60-64 | Spellslayer |
| 25-29 | Silent Hand | 65-69 | Grandmaster of Shadows |
| 30-34 | Shade | 70+ | Assassin |

**Its spells,** the stock Mage-1 line: magic missile (1), frost jet (4), lightning bolt (8),
sunbolt (13), eldritch bolt (20).

## Shared rules

- **Stealth.** Both have the stock Class Stealth ability, so they can SNEAK and HIDE like the
  other stealth classes, and their stealth grows the stock way.
- **Backstab.** Both get stock backstab damage abilities (`BackstabMin` / `BackstabMax`),
  smaller than a Thief's (10) since they also cast: proposed 5 each.
- **No thievery, traps or lockpicking.** They're killers, not burglars; the Thief, Gypsy
  and Missionary keep those.
- **No Spellstrike**, ever (stealth classes).

## Spell backstab (the Assassin)

Stock backstab works only with a melee blow, from hiding or sneaking. The Assassin can open
with a spell instead:

- **When:** casting a single-target attack spell at a monster (or player) from hiding or
  sneaking, at a target that isn't already fighting the Assassin: the opening cast of a
  fight, the spell's equivalent of BACKSTAB.
- **What it does:** the spell's damage gets the caster's backstab bonus: it's raised by a
  percentage (a fork class ability, `SpellBackstab`, proposed 100: double damage), and the
  caster's `BackstabMin` / `BackstabMax` are added, as a backstab blow's are.
- **It still has to be cast:** the stock cast roll applies, and a failed cast is a failed
  ambush; the Assassin is revealed either way.
- **After it:** the Assassin is out of hiding and the fight goes on as normal (auto-combat
  with the spell, or melee).
- **Messages:** a lead-in like "You strike from the shadows with magic missile!" and the
  room's version, before the stock cast lines.

How it works with stock: the cast path learns one new condition, "cast from stealth by a
class with `SpellBackstab`, at a target not fighting the caster", and applies the damage
bonus to the rolled amount. Everything else is the stock cast.

## Vanish (the Avenger)

Stock HIDE always fails in a fight. The Avenger can **VANISH** instead:

- **Command:** `VANISH` (new). Only for a class with the fork ability `Vanish`.
- **When:** in combat (stock HIDE covers the rest). It costs the round: the Avenger makes no
  blows that round.
- **The roll:** the stock hide chance, with a penalty for fighting (proposed −20), at most
  95.
- **On success:** the Avenger breaks off auto-combat and is hidden. Monsters fighting it lose
  it, as they lose a player who leaves; it can then BACKSTAB (or cast) again as an opener.
- **On failure:** "You try to slip away, but your foe keeps you in sight." The round is
  spent and the fight goes on.
- **Cooldown:** once every few rounds (proposed: 5 combat rounds), so it's a rhythm, not a
  permanent invisibility.

**Blunt backstab.** A stock backstab with a weapon needs "Alter Backstab Attack Value" on
the weapon, which blunt weapons rarely have, so an Avenger would almost never backstab.
A fork class ability, `BluntBackstab`, lets the class backstab with any blunt weapon it can
wield, as if it carried that ability. Without it, vanish has nothing to set up.

## Fitting it into silvermere

| Piece | Where |
|---|---|
| Both classes, titles and HELP | content (`content/classes.toml`, `content/help.toml`), like the War Mage and the Warden |
| Fork abilities `SpellBackstab`, `Vanish`, `BluntBackstab` | `silvermere-types` `fork` |
| Spell backstab | one hook in the cast path where an attack spell's amount is rolled |
| VANISH | a new command (an input hook like `PET`), using the stock hide roll and auto-combat break-off |
| Blunt backstab | one condition in `cmd_backstab` |
| Vanish cooldown | a per-session counter in the engine |

## Decisions

1. Two fighter-leaning stealth casters: fighting 3, magic level 1, leather, 4-8 HP, 200%.
2. The Avenger is Priest-1 with blunt weapons; the Assassin is Mage-1 with sharp weapons.
3. Signatures: the Assassin's spell backstab, the Avenger's vanish.
4. Neither gets Spellstrike (stealth classes).

## Open questions

1. **Titles and lore:** the proposals above, for you to edit or replace.
2. **Spell backstab's strength:** double damage plus `BackstabMin` / `BackstabMax`, or
   something else? Against players too, or monsters only?
3. **Vanish's numbers:** the −20 penalty and the 5-round cooldown.
4. **Blunt backstab:** add it (so the Avenger can backstab after vanishing), or keep stock
   backstab rules and let vanish be an escape?
5. **Thievery, traps and picklocks:** none for either, as proposed, or some?
6. **Class numbers:** 18 (Avenger) and 19 (Assassin), titles in blocks 3021 and 3022.

## Brainstorm

- Assassin-only poisons on its blades, through the stock poison mechanic.
- An Avenger "mark": the target named by its temple takes extra damage from it.
- Trainers for both, in the room renovations after the Warden.
