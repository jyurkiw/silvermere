# Battle-mage spells (sketch)

Status: **preliminary sketch, not settled.** No numbers yet; those come with the balance
sim ([testing.md](testing.md)). Fork-only. Kept on its own branch, unmerged.

- [Summary](#summary)
- [Who gets them](#who-gets-them)
- [What stock already has](#what-stock-already-has)
- [Strikes: Spellstrike-native attack spells](#strikes-spellstrike-native-attack-spells)
- [Melee spells: weapon enchantments](#melee-spells-weapon-enchantments)
- [The flametongue](#the-flametongue)
- [What needs code](#what-needs-code)
- [Content work](#content-work)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

New spells shared by the **War Mage** and the **Warlock**, the two halves of the King's
Battle-Mage corps ([war-mage.md, Lore](war-mage.md#lore)). There are two families:

- **Strikes**: attack spells made to be cast through a weapon with Spellstrike. They're
  cheap, easy to land, and meant to go out round after round.
- **Melee spells**: buffs that enchant the caster's weapon so that every hit deals extra
  elemental damage. These replace the warlock's flametongue.

Mages never get them: the two classes are close cousins in a way a mage isn't.

## Who gets them

Stock gates spells by magic type and level only, so a Mage-1 spell could be cast by any
Mage, Warlock, Gypsy or War Mage. Spells are learned from **scrolls**, though (stock
examples: *scroll of lightning bolt*, *scroll of eldritch bolt*), and a scroll can carry a
class list. Every new spell's scroll lists **War Mage and Warlock** only.

- The spells are Mage magic. The early ones are Mage-1, so the War Mage can cast them;
  Warlock-only tiers, if any, are Mage-2.
- Stealth classes never get them (Gypsies are Mage-2 but stealth;
  [war-mage.md, Decision 13](war-mage.md#decisions)).
- Scroll class lists stop Mages *learning* them. Whether a Mage who somehow has one in
  its spellbook (a sysop edit) can cast it is [open question 1](#open-questions).

## What stock already has

The War Mage's stock single-target attack spells, by level: magic missile (1), frost jet
(4), lightning bolt (8), sunbolt (13), eldritch bolt (20, stops growing at 28). The
Warlock adds meteor swarm (25), dragonfire (30) and colour spray (32). Meteor swarm costs
250 energy, so a stock casting round fires it up to 4 times.

Stock spell effects the new spells can reuse as data, with no code:

| Effect | Stock example | Use here |
|---|---|---|
| Damage, armour applies (`DamageArmour`) | magic missile, eldritch bolt | Strikes |
| An element on the spell (cold, fire, lightning, stone, water, poison) | frost jet (cold), sunbolt (fire) | Strikes, against monster resistances |
| Extra damage on the caster's blows (`Damage`) | smite (Mage-1, level 3) | |
| A spell cast at the target on every hit | the flametongue's *flame hits* | Melee spells |
| Better crits (`Critical`) | frenzy | |
| Better hit chance (`AvBless`) | bless-type spells | |
| Faster blows (`EnergyUse`) | speed | |
| Damage back to attackers (`DamageShield`) | shockshield | |

## Strikes: Spellstrike-native attack spells

What makes a strike different from a stock attack spell:

- **Cheap and castable.** Low mana and a mild difficulty, so a War Mage can cast one every
  round (the *Conserve* run of the balance sim) without its cast chance collapsing.
- **Modest damage, steady growth.** Less than a full-round spell of the same level, since
  the blows still land beside it, and it keeps growing with level instead of stopping
  early.
- **An element each.** A spread of elements across the ladder, so the battle-mage picks
  the strike a monster is weak to.
- **Through the weapon.** In the lore a strike is channelled through the blade, not
  thrown. Whether that's enforced (a strike can only be cast as a spellstrike) is
  [open question 2](#open-questions).

A first sketch of the ladder. Names, levels and order are placeholders:

| Rough level | Strike | Element | Idea |
|---|---|---|---|
| 3 | **spark strike** | lightning | the first strike, cheaper than magic missile |
| 8 | **rime strike** | cold | a frost jet you can afford every round |
| 14 | **ember cleave** | fire | |
| 20 | **stonebreaker** | stone | hits hard against armoured targets |
| 28 | **stormcall strike** | lightning | where eldritch bolt stops growing |
| 36 | **corps' wrath** | normal | the top strike, no element to resist |

## Melee spells: weapon enchantments

The warlock's and War Mage's **melee spells** are buffs the caster puts on their own
melee weapon. While one lasts, **every hit also deals extra elemental damage**: fire,
cold, lightning and so on, checked against the target's resistance to that element. A
selection of them, one per element, gives the battle-mage a choice against each monster.

**How it works, using what stock already has.** Stock weapons can already deal elemental
damage on every hit: the *flametongue* (below) carries an on-hit spell, *flame hits*
(1-3 fire damage, armour doesn't apply), which is cast at the target each time the blade
connects. A melee spell does the same thing temporarily: while it lasts, each hit casts the
enchantment's on-hit spell at the target. The on-hit spells are ordinary spell records, so
their damage, element and growth with level are data. What needs code is letting a buff
on the caster, not only the weapon item, carry the on-hit spell.

A first sketch. Names, levels and order are placeholders:

| Rough level | Melee spell | Element | Extra damage per hit |
|---|---|---|---|
| 5 | **searing brand** | fire | small, grows with level |
| 10 | **frost brand** | cold | as above |
| 15 | **storm brand** | lightning | as above |
| 20 | **stone brand** | stone | as above |
| 30 | **venom brand** | poison | as above |
| 35 | **corps' brand** | normal | no element to resist, the top of the line |

Rules sketched so far:

- **Self only**, and only with a weapon wielded. It ends when the weapon is unwielded.
- **One at a time.** Casting a second replaces the first (stock buffs already replace each
  other through their "removes spell" lists, so that part is data).
- **Casting one is a stock casting round**, not a spellstrike: a melee spell isn't an
  attack spell. Giving up a round's blows buys several rounds of better ones.
- **It applies to every hit,** so it rewards fast weapons and many blows, the opposite
  pull from a two-hander's big single hits.
- Other buffs that are data only, if wanted: crit chance (`Critical`), hit chance
  (`AvBless`).

## The flametongue

The stock warlock class weapon, item 352: a one-handed sharp blade, speed 1400, 4-18
damage, Strength 45, minimum level 10, a limited item (one copy in the realm), warlock
only. Besides +2 Spellcasting, more mana regeneration and light, it carries *flame hits*,
fire damage on every hit. It lies in a Large Tomb, room 1/2301 of the crypt, placed there
so that a new copy appears when none is lying in the room.

The melee spells do what the flametongue does, so the flametongue is redesigned:

1. **Retired.** It's no longer placed in room 1/2301. Existing copies are a server question
   (a fresh fork server has none; see [open question 8](#open-questions)).
2. **Replaced in the crypt** by a new warlock class weapon, placed in room 1/2301 in its
   place. It's built around the melee spells instead of a fixed fire: for example, it makes
   melee spells last longer or hit harder, keeps the Spellcasting bonus and the light, and
   has no element of its own. It stays warlock only, unless War Mages get a class weapon
   too ([open question 9](#open-questions)).

## What needs code

| Feature | Needs code? |
|---|---|
| The spells themselves, their scrolls and their class lists | no, content only (with the overlay grown to spells and items) |
| The on-hit spells (element, damage, growth) | no, they're spell records |
| A melee spell on the caster casting its on-hit spell at every hit | yes: today only a weapon item can |
| A melee spell ending when the weapon is unwielded | yes |
| A strike that can only be cast as a spellstrike | yes: a new spell ability, checked at cast |
| Stopping a Mage casting a spell its class can't learn | yes, if wanted |
| Retiring the flametongue and placing its replacement | no, content (a room change and a new item) |

## Content work

The overlay's first version covers classes and HELP topics only
([content-overlay.md, Decision 4](content-overlay.md#decisions)). These spells need it to
grow to cover:

- **spells**: new spell records;
- **items**: the scrolls, with their class lists, and the flametongue's replacement;
- **rooms**: a change to room 1/2301's placed items;
- **shop stock**: where the scrolls are sold (a battle-mage guild shop, or added to an
  existing magic shop);
- **messages**: the cast and effect lines of each new spell.

## Open questions

1. **Casting outside the class list.** Should a class that can't learn a spell also be
   unable to cast it, or are scroll class lists enough?
2. **Strikes only through a weapon.** Should strikes be castable only as spellstrikes (needs
   a new spell ability), or also as ordinary ranged spells?
3. ~~Elemental melee spells.~~ **Decided:** melee spells deal real elemental damage on every hit, through on-hit spells.
4. **Warlock-only tiers.** Are any strikes or melee spells Mage-2, warlock only?
5. **The 20+ gap.** The War Mage's stock attack spells stop at eldritch bolt. Do strikes
   fill that, or is a separate high-level spell wanted?
6. **Meteor swarm and Spellstrike.** Meteor swarm costs a quarter of a round, so a stock
   warlock can fire it 4 times a round. A spellstrike casts once. When warlocks get
   Spellstrike, should cheap spells cast more than once in a spellstrike round?
7. **Where scrolls come from.** Sold in a shop (which one), dropped, or both?
8. **Existing flametongues.** On a server that already has one in play, does it stay as it
   is, turn into the new weapon, or vanish?
9. **A War Mage class weapon.** Does the War Mage get a class weapon of its own (a
   two-hander, say), or share the warlock's replacement?

## Brainstorm

- A battle-mage guild in Silvermere with its own scroll shop and trainer, tied to the
  King's Battle-Mage corps.
- A strike that heals the caster a little on a kill.
- Melee spells that combine with Spellstrike: a fire brand plus a fire strike for a bonus.
