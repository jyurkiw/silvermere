# The Warden, and pets

Status: **draft, in design.** Nothing here is implemented. Fork-only. This changes
silvermere's own code more than the War Mage did (a pet table, a pet's blows in the combat
round, pet names in output), and that's accepted for this one.

- [Summary](#summary)
- [The pet model](#the-pet-model)
- [How the model covers each case](#how-the-model-covers-each-case)
- [The Warden](#the-warden)
- [Level titles](#level-titles)
- [The pet's attack](#the-pets-attack)
- [The Newhaven stable: choosing a companion](#the-newhaven-stable-choosing-a-companion)
- [Naming the companion](#naming-the-companion)
- [The pet in output text](#the-pet-in-output-text)
- [Weapons: two-handed and ranged](#weapons-two-handed-and-ranged)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

The **Warden** wanders the realm with an animal companion. In the first version the
companion is mostly presentation: it shares the Warden's hit points, has no inventory and
wears nothing, and monsters fighting the Warden are fighting "both of you" (which, in the
game's terms, is just the Warden). What the pet adds is **its own blows every round**, at
its own speed and damage, scaling with the Warden's level, costing no mana. The Warden
fights in leather with two-handed or ranged weapons.

The pet support is built as **general pet support**, not a Warden special case, so later
pet classes and pet spells (a summoned flame falcon, a Warden 2.0 whose companion has its
own hit points and gear) use the same table and the same combat hook.

## The pet model

Two tables: pet **kinds** are content, each character's **pets** are play state.

**`pet_kinds`** (content, installed from TOML at every start, like classes):

| Field | Meaning |
|---|---|
| `kind` | number and name: wolf, bear, hawk, snake, flame falcon, ... |
| attack | weapon speed, damage range, accuracy, and how damage and accuracy grow with the owner's level |
| `element` | the element of its damage, if any (a flame falcon's fire) |
| wording | the attack's verbs ("bites", "rakes", "strikes") and its miss wording |
| `own_hp` | false: shares the owner's hit points (a Warden's companion). True: has its own, and becomes something monsters can attack (a summoned falcon, Warden 2.0) |

**`pets`** (state, one row per pet a character has):

| Field | Meaning |
|---|---|
| `id`, `owner` | the pet, and the character who owns it |
| `kind` | a `pet_kinds` entry |
| `name` | the pet's name ("Fang") |
| `source` | where it came from: the class, a spell, an item, a quest |
| `summoned` | out with its owner now. Only summoned pets fight and show in text |
| `immortal` | can't die; `dead` is ignored |
| `dead` | killed; fights no more until raised (only meaningful when not immortal) |
| `expires` | for a summon, when it goes away (rounds or a time); none for a companion |
| `hp` | its own hit points, when its kind has them |

## How the model covers each case

| Case | `summoned` | `immortal` | `dead` | Notes |
|---|---|---|---|---|
| **Warden's companion** (first version) | true | true | ignored | shares HP, never dies, always out |
| **A Warden with several pets** (wolf, bear, hawk, snake) | true for the active one, false for the rest | true | ignored | switching companions flips `summoned` |
| **Flame falcon spell** | true | false | false | its own HP; attacks the caster's target for a little extra damage; monsters can attack it; gone when killed or when `expires` passes |
| **Warden 2.0** | true | false | true/false | its own HP and gear; while `dead`, the Warden walks as though heavily encumbered, until it's raised |

## The Warden

Decided: a tough Warden that swings less often, with a slow companion that hits hard.

| Field | Warrior | Warden | Note |
|---|---|---|---|
| class number | 1 | 17 | the next after the War Mage |
| HP per level | 6-10 | **7-11** | more than any stock class |
| fighting | 4 (`combat` 6) | **3** (`combat` 5, "Average") | fewer swings of its own |
| armour | plate | **leather; chain from level 10; scale from level 20** | see below |
| weapons | any | two-handed and ranged only | see [Weapons](#weapons-two-handed-and-ranged) |
| magic | none | none | its pet is not a spell |
| experience | 100% | **140%** | the Priest's and Mage's, just above the Witchunter (130%); you set it around 130-140%; the balance sim checks it |
| abilities | Bash | Bash; a companion at creation; the armour steps | fork abilities |
| titles | | text block 3020 | [below](#level-titles) |

**Armour by level.** A class record holds one armour limit, so armour that improves with
level is a new fork rule, the same kind as `NoOffHand`: two fork abilities,
`ChainmailAtLevel` (10) and `ScalemailAtLevel` (20), checked where gear is allowed. Below
level 10 the limit is the class's own (leather); from 10 chain; from 20 scale.

A new Warden bonds its companion at the Newhaven stable
([below](#the-newhaven-stable-choosing-a-companion)): `source` = class, `immortal`,
`summoned`.

### Level titles

Sixteen ranks, in the same level bands as every stock class, the same title for either
sex. Huntsman runs to level 110 like the stock last ranks. The block is a new text block,
3020, after the War Mage's 3019.

| Levels | Title | Levels | Title |
|---|---|---|---|
| 1 | Cub | 35-39 | Pioneer |
| 2-4 | Scout | 40-44 | Frontiersman |
| 5-9 | Tenderfoot | 45-49 | Voyageur |
| 10-14 | Venturer | 50-54 | Long Hunter |
| 15-19 | Star Scout | 55-59 | Pathfinder |
| 20-24 | Life Scout | 60-64 | Bush Ranger |
| 25-29 | Explorer | 65-69 | High Tracker |
| 30-34 | Rover | 70-110 | Huntsman |

## The pet's attack

**The Warden's companion: one heavy blow a round.** "Attacks less, but you feel
it": rather than an energy pool, the companion makes exactly one blow each round, after
the Warden's, at the Warden's target. All the scaling is in its damage:

| Warden level | Companion's blow (proposed) |
|---|---|
| 1 | 2-10 |
| 10 | 7-28 |
| 20 | 12-48 |
| 30 | 17-68 |

Damage is `2 + level/2` to `8 + 2 × level`, rolled to hit against the target's armour
through the stock blow resolution (a pet's blow is a fighter's blow). These are placeholders
for the balance sim to tune; they're values on the pet kind, not code.

General rules for any pet:

- only **summoned, living** pets attack;
- **no mana, no casting, no cost to the owner**: the owner's energy isn't touched;
- **its own lines**: "Fang bites the giant rat for 9 damage!", and its own miss line;
- a kind sets **blows per round**: a fixed count (the Warden's companion: 1), or an energy
  pool at its weapon speed for pets that should strike more often as they're quicker
  (a later falcon, say).

The first version's companion shares its owner's hit points, so monsters attack only the
owner, as they do today. A pet with its own hit points (falcon, Warden 2.0) is a later
step: it needs monsters able to choose it as a target.

## The Newhaven stable: choosing a companion

A new Warden chooses its companion in the game world, not on the creation screens: its first
stop is a **stable** in Newhaven.

- **New rooms in Newhaven** for the stable, reached by an exit from an existing Newhaven
  room.
- **A stablemaster NPC**, a permanent monster in the stable. Players can `ask stablemaster
  about wardens` (and about each companion) for the lore and the choices. That's the stock
  talk mechanism: the NPC's record names a talk text block of `KEYWORDS:text` lines.
- **The stable is a shop.** `LIST` shows four **leashes**, each priced at nothing:

| Leash | Companion | Character |
|---|---|---|
| hound's leash | **hunting hound** | average: the baseline companion |
| hawkling's leash | **hawkling** | a low-level bird: strikes several times a round, little damage each |
| rattlesnake's leash | **trained rattlesnake** | low damage, but its bite poisons |
| drop bear's leash | **Drop Bear** | slow (a blow every other round) but hits very hard |

- **Bonding is a room script, in the stable.** Stock rooms can react to what players type
  there and run script clauses, and those already cover bonding: check the player's class,
  check for and take an item, add or remove an ability, show text.  You buy (or are given) a **leash** and **use it in the
  stable**: stock `USE` offers the line to the room's script first, so `use hound's leash`
  there runs:

  ```
  class is Warden? -> takeitem hound's leash -> removeability PetKind ->
  addability PetKind 1 -> text "You bond with your hunting hound."
  ```

  Used anywhere else, a leash does nothing special.

  The companion's kind lives on the character as a fork ability, `PetKind`, which saves
  with the character like any ability; the combat code reads it to know which companion
  fights. **No new code for bonding.** A Warden who changes their mind comes back for
  another leash and bonds again.
- **All pet acquisition is content:** shops (like the stable) or retrieval quests whose
  script ends with the same clauses. A quest that rewards a new companion needs no code.
- Until a Warden bonds a companion, it fights alone.

Proposed companion stats (placeholders for the balance sim; values on each pet kind):

| Kind | Blows | Damage per blow, by Warden level (1 / 10 / 20 / 30) | Extra |
|---|---|---|---|
| hunting hound | 1 a round | 2-10 / 7-28 / 12-48 / 17-68 | |
| hawkling | about 3 a round (an energy pool at a fast speed) | 1-4 / 2-9 / 4-16 / 5-22 | strikes often, so it lands more consistently |
| trained rattlesnake | 1 a round | 1-5 / 4-14 / 6-24 / 9-34 | poisons the target |
| Drop Bear | 1 every other round (a slow energy pool) | 4-20 / 14-56 / 24-96 / 34-136 | |

Aim: about the same damage over a fight for each, delivered differently. The sim checks
it.

## Naming the companion

A new command, `PET`:

- `PET NAME Fang` names the summoned companion. A name is one word of letters, up to 10
  characters (the same limit as a character's first name), capitalised as typed. It can be
  changed any time.
- `PET` alone shows the companion: its name, kind and what it does.
- Until it's named, a companion goes by its kind: "your hunting hound", "Devin and his
  hunting hound".

The name is kept in the companion's `pets` row. It's display only: players still target
the Warden, not the pet.

## The pet in output text

One helper gives a character's displayed name with its summoned pet, "Devin and Fang", used
where it reads well:

- the who list: "Devin and Fang – Apprentice …";
- arrivals and departures: "Devin and Fang just walked in from the north.";
- the pet's own combat lines.

**Targeting is unchanged.** Players target the character, "Devin", only; typing the pet's
name does nothing. That oddity is accepted for now.

**Left plain for now:** the room's "Also here:" line. MegaMud reads it to track players,
and "Devin and Fang" there could confuse its player and party tracking. To revisit after a
MegaMud test.

## Weapons: two-handed and ranged

Two-handed is a stock weapon class (`AnyTwoHand`). Stock has no ranged combat: slings,
bows and crossbows are ordinary weapon items with missile names and weapon types (a sling
is one-handed blunt, a crossbow two-handed sharp). So "ranged" is marked on the items:

- **A fork item ability, `Ranged`.** Content marks the stock slings, bows, crossbows and
  other missile weapons with it (an item change in the overlay; the missile weapon shops,
  such as Jael's Missile Weapons, Rhudaur's and Arlysia's, list most of them).
- **A fork class ability, `RangedWeapons`.** A class with it may use any item marked
  `Ranged`, whatever the item's weapon type, on top of its own weapon class.
- **The Warden's weapon class is `AnyTwoHand`**, plus `RangedWeapons`: every two-hander,
  and every ranged weapon, even a one-handed sling.

Both abilities are general: a later archer class, say, reuses them.

**For now `Ranged` is only a marker.** It decides who may use the weapon and nothing else:
a ranged weapon still fights exactly like the melee item it is in stock, with no range, no
ammunition and no special rules. Real ranged combat can hang off the same marker later.

## Fitting it into silvermere

| Piece | Where |
|---|---|
| `pet_kinds` | content (`content/pets.toml`), installed by the overlay, grown to cover pet kinds |
| Armour by level | two fork abilities checked in `items/can_use.rs` |
| Ranged weapons | the fork item ability `Ranged` (set on stock items by content) and the class ability `RangedWeapons`, checked in `items/can_use.rs` |
| `pets` table | a migration (SQLite and PostgreSQL), a repository in `silvermere-db`, `docs/schema.md` regenerated |
| Pets in the world | loaded with the characters, saved through the persistence worker when they change |
| The pet's blows | `silvermere-engine`, after the owner's blows in the round; a pet fighter built in `silvermere-rules` from its kind |
| Displayed name | one helper in the engine, used by the who list and the arrival and departure lines |
| The Warden class | content, like the War Mage |
| `PET` / `PET NAME` | a new command in the engine; the name in the `pets` row |
| The stable | content: new Newhaven rooms, the stablemaster (a talking monster and its text blocks), the stable shop, four leash items. Needs the overlay to grow to rooms, monsters, shops, items and text blocks |
| Bonding | room scripts (stock clauses) setting the fork character ability `PetKind`; no code |
| `PetKind` | read by the pet's attack to pick the companion's kind |

## Decisions

1. Pets get their own table, not a hiding place in the character record.
2. Pet kinds are content; each character's pets are state.
3. Pets carry `summoned`, `immortal` and `dead` flags (plus source, expiry and optional HP),
   so companions, rosters of companions, summons and mortal pets are one model.
4. The Warden's first companion is immortal, shares its owner's HP, and is always
   summoned.
5. Targeting stays on the character only.
6. The Warden wears leather (chain from level 10, scale from 20) and uses two-handed or
   ranged weapons.
8. The companion is chosen in game, at a new stable in Newhaven: a stablemaster to ask, and
   four free leashes (hound, hawkling, rattlesnake, Drop Bear). You use the leash in the
   stable, whose room script takes it and sets the character's `PetKind` ability.
12. Ranged weapons are marked with a fork `Ranged` item ability; the Warden's
    `RangedWeapons` class ability allows them beside its two-handers.
11. The companion is named with `PET NAME`; `PET` shows it.
10. Titles: Cub to Huntsman, sixteen ranks, in text block 3020.
9. All pet acquisition is content: shops or retrieval quests, ending in the same script
   clauses. No code for acquiring pets.
7. The Warden has high HP and low fighting (3): it swings less, and its companion hits
   hard. 7-11 HP, 140% experience, one companion blow a round.

## Open questions

1. ~~Stats.~~ **Decided:** 7-11 HP, fighting 3, 140% experience, leather then chain (10)
   and scale (20), and a companion with one heavy blow a round (damage to be tuned).
2. ~~Pet kinds at launch.~~ **Decided:** a choice of four at the Newhaven stable: hunting
   hound, hawkling, trained rattlesnake, Drop Bear, each different in speed, damage or
   effect.
3. ~~Naming.~~ **Decided:** a `PET NAME` command after bonding
   ([Naming the companion](#naming-the-companion)).
4. ~~Ranged weapons.~~ **Decided:** a fork `Ranged` item ability, and a `RangedWeapons`
   class ability that allows those items.
5. **How strong is the pet?** Its damage per round against a Warrior's at the same level is
   the balance lever; the balance sim ([testing.md](testing.md)) can measure it.
6. **The "Also here:" line:** show "Devin and Fang" there after a MegaMud test, or never?

## Brainstorm

- **Flame falcon** (a pet spell): a mortal summon with its own HP that attacks the caster's
  target until killed or until it expires.
- **Warden 2.0**: the companion has its own HP and equipment; while it's dead, the Warden
  walks as though heavily encumbered until it's raised.
- Quests, vendors and items that change the active companion's kind, or add kinds to a
  Warden's roster.
- Pets reaching the who list, the room line and other text step by step, with MegaMud
  checked at each step.
