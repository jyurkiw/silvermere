# The Warden, and pets

Status: **draft, in design.** Nothing here is implemented. Fork-only. This changes
silvermere's own code more than the War Mage did (a pet table, a pet's blows in the combat
round, pet names in output), and that's accepted for this one.

- [Summary](#summary)
- [The pet model](#the-pet-model)
- [How the model covers each case](#how-the-model-covers-each-case)
- [The Warden](#the-warden)
- [The pet's attack](#the-pets-attack)
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

| Field | Value | Note |
|---|---|---|
| name | Warden | |
| class number | 17 | the next after the War Mage |
| HP per level | *open* | |
| experience | *open* | |
| magic | none | the Warden casts nothing; its pet is not a spell |
| fighting | *open* | |
| weapons | two-handed and ranged only | see [Weapons](#weapons-two-handed-and-ranged) |
| armour | leather | |
| abilities | a fork ability marking the class as starting with a companion | |
| titles | *open* | |

A new Warden starts with one companion (`source` = class, `immortal`, `summoned`) of a kind
to be decided, and names it ([open question 3](#open-questions)).

## The pet's attack

Each round, for every **summoned, living** pet a fighting character has, the pet makes its
own blows at the character's target, after the character's:

- **its own energy**: a full round (1000), spent on blows at the pet's weapon speed, so a
  fast pet strikes more often;
- **to-hit and damage** from its kind, grown with the owner's level, against the target's
  armour, through the stock blow resolution (a pet's blow is a fighter's blow);
- **no mana, no casting, no cost to the owner**: the owner's energy isn't touched;
- **its own lines**: "Fang bites the giant rat for 9 damage!", and its own miss line.

The first version's companion shares its owner's hit points, so monsters attack only the
owner, as they do today. A pet with its own hit points (falcon, Warden 2.0) is a later
step: it needs monsters able to choose it as a target.

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
is one-handed blunt, a crossbow two-handed sharp). So "ranged" has to be marked:

- **a fork item ability**, `Ranged`, on those items (set by content), with a Warden rule
  that allows two-handed weapons or `Ranged` ones; or
- **Class Item Inclusion** for the Warden on each one-handed ranged item, with no new rule.

[Open question 4](#open-questions).

## Fitting it into silvermere

| Piece | Where |
|---|---|
| `pet_kinds` | content (`content/pets.toml`), installed by the overlay, grown to cover pet kinds |
| `pets` table | a migration (SQLite and PostgreSQL), a repository in `silvermere-db`, `docs/schema.md` regenerated |
| Pets in the world | loaded with the characters, saved through the persistence worker when they change |
| The pet's blows | `silvermere-engine`, after the owner's blows in the round; a pet fighter built in `silvermere-rules` from its kind |
| Displayed name | one helper in the engine, used by the who list and the arrival and departure lines |
| The Warden class | content, like the War Mage |
| Naming | creation or a command ([open question 3](#open-questions)) |

## Decisions

1. Pets get their own table, not a hiding place in the character record.
2. Pet kinds are content; each character's pets are state.
3. Pets carry `summoned`, `immortal` and `dead` flags (plus source, expiry and optional HP),
   so companions, rosters of companions, summons and mortal pets are one model.
4. The Warden's first companion is immortal, shares its owner's HP, and is always
   summoned.
5. Targeting stays on the character only.
6. The Warden wears leather and uses two-handed or ranged weapons.

## Open questions

1. **Stats:** the Warden's HP per level, experience cost and fighting value.
2. **Pet kinds at launch:** one kind for every Warden, or a choice of a few (wolf, bear,
   hawk, snake)? Do kinds differ only in flavour, or in speed and damage too?
3. **Naming:** at character creation (touches the creation screens) or a `PET NAME` command
   afterwards (simpler)?
4. **Ranged weapons:** a `Ranged` item ability, or Class Item Inclusion per item?
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
