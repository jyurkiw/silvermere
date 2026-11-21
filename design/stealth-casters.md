# The Avenger and the Assassin: stealth casters

Status: **draft, in design.** Nothing here is implemented. Fork-only.

- [Summary](#summary)
- [Where they sit](#where-they-sit)
- [The Avenger](#the-avenger)
- [The Assassin](#the-assassin)
- [Shared rules](#shared-rules)
- [Spell backstab](#spell-backstab)
- [Vanish (the Avenger)](#vanish-the-avenger)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

Two fighter-leaning stealth casters. Unlike the Gypsy and the Missionary (fighting 2,
magic level 2), they fight first and cast second: fighting 3, magic level 1.

- **Both** can **spell backstab**: open from hiding with an attack spell that lands like a
  backstab, if they keep their stealth and land the cast.
- **The Avenger** (Priest-1, blunt weapons): the hidden hand of the temples. It can also
  **vanish**: drop back into hiding in the middle of a fight, to strike again from the
  shadows. (Backstabbing with godstrike is the point.)
- **The Assassin** (Mage-1, sharp weapons): an arcane killer, whose craft is the opening
  spell.

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
| abilities | stealth, backstab, **spell backstab**, **vanish**, **blunt backstab** | see [Spell backstab](#spell-backstab), [Vanish](#vanish-the-avenger) |
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
| abilities | stealth, backstab, **spell backstab** | see [Spell backstab](#spell-backstab) |
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

## Spell backstab

Stock backstab works only with a melee blow, from hiding or sneaking. Both classes can open
with a spell instead:

- **When:** casting a single-target attack spell from hiding or sneaking at a target (a
  monster or, under PvP's usual rules, a player) that isn't already fighting the caster: the opening cast of a fight, the spell's equivalent of
  BACKSTAB. One attempt: whatever happens, the caster is revealed and the fight is on.
- **Two rolls, independent of each other:**
  - **the stealth roll**: did the caster stay unseen until the spell left their hands? The
    stock sneak/hide chance, with a penalty for the casting (proposed −20, tunable as the
    `SpellBackstab` ability's companion value);
  - **the cast roll**: the stock cast chance, unchanged.
- **The outcomes:**

| Stealth | Cast | Result |
|---|---|---|
| kept | lands | **spell backstab**: the spell's damage raised by the class's `SpellBackstab` percentage (proposed 100: double) plus its `BackstabMin` / `BackstabMax` |
| lost | lands | a normal cast: normal damage |
| kept | fails | a failed cast: the ambush is wasted |
| lost | fails | a failed cast |

- **Why it's fair:** a Mage-1 or Priest-1 caster with modest Intellect or Willpower won't land
  both rolls often until the higher levels, so the big damage is worth it when it comes.
- **Messages:** a lead-in like "You strike from the shadows with godstrike!" (and the room's
  version) when stealth is kept, before the stock cast lines; when stealth is lost, "Your foe
  spots you as you begin to cast!" and then a normal cast.

How it works with stock: the cast path learns one new condition, "cast from stealth by a
class with `SpellBackstab`, at a target not fighting the caster", makes the stealth roll, and
applies the damage bonus to the rolled amount when both rolls succeed. Everything else is
the stock cast.

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

**Blunt backstab** (decided, for flavour). A stock backstab with a weapon needs "Alter
Backstab Attack Value" on the weapon, which blunt weapons rarely have. A fork class ability,
`BluntBackstab`, lets the Avenger backstab with any blunt weapon it can wield, as if it
carried that ability. The Avenger's main opener is still the spell backstab; this lets a
mace do the same job.

*If blunt backstab turns out awkward*, the fallback is **SAP**: a separate command that works
only with blunt, non-ranged weapons (the `Ranged` marker the Warden added) and is otherwise
exactly BACKSTAB.

## Fitting it into silvermere

| Piece | Where |
|---|---|
| Both classes, titles and HELP | content (`content/classes.toml`, `content/help.toml`), like the War Mage and the Warden |
| Fork abilities `SpellBackstab`, `Vanish`, `BluntBackstab` | `silvermere-types` `fork` |
| Spell backstab | one hook in the cast path where an attack spell's amount is rolled, with its own stealth roll |
| VANISH | a new command (an input hook like `PET`), using the stock hide roll and auto-combat break-off |
| Blunt backstab | one condition in `cmd_backstab` |
| Vanish cooldown | a per-session counter in the engine |

## Decisions

1. Two fighter-leaning stealth casters: fighting 3, magic level 1, leather, 4-8 HP, 200%.
2. The Avenger is Priest-1 with blunt weapons; the Assassin is Mage-1 with sharp weapons.
3. Both have spell backstab: a stealth roll and a cast roll, independent, one attempt.
   Stealth kept and cast landed: backstab damage; stealth lost: a normal cast.
4. The Avenger also has vanish (−20 to the hide chance, every 5 rounds; to be balance
   tested) and blunt backstab (with SAP as the fallback).
5. Neither has thievery, traps or picklocks; those stay with the Thief, Ninja, Missionary
   and Gypsy.
6. Neither gets Spellstrike (stealth classes).
7. The proposed titles and lore stand (they can change later).
8. Spell backstab works against players too, under PvP's usual rules.

## Open questions

1. ~~Spell backstab against players.~~ **Decided:** players too, under PvP's usual rules
   (evil points, lawful and level-range checks, as any attack spell at a player). MajorMUD was
   never balanced for PvP, so PvP balance isn't a goal here.
2. **The numbers to balance-test:** spell backstab's +100% and −20 stealth penalty, vanish's
   −20 and 5 rounds, the backstab bonus of 5.
3. **Class numbers:** 18 (Avenger) and 19 (Assassin), titles in blocks 3021 and 3022.

## Brainstorm

- Assassin-only poisons on its blades, through the stock poison mechanic.
- An Avenger "mark": the target named by its temple takes extra damage from it.
- Trainers for both, in the room renovations after the Warden.
