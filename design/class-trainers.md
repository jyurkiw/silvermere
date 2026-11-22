# Class trainers, the tomb quests and class weapons

Status: **implemented** (`feature/class-trainers`). Fork-only.

**As built,** where it differs from the plan below:

- **The Temple's catch and lever** use stock's own ordered puzzle, not a fork ability: the
  foyer's north exit is a hidden exit with two blocks that must go in order, and `push
  catch` and `pull lever` are two remote-action exits (in the foyer's free southeast and
  southwest slots) that each take one block. The lever alone does nothing. Once both are
  gone, the door shows for anyone for a while, as stock's push-button passage does.
- **The class weapons are class-only through the item's class list.** Stock's "Class
  Item" ability *lets* a class use an item it otherwise couldn't; it doesn't restrict it.
- **The tombs and wing passages are dark,** as stock's are (light −175): a character needs a
  light there, as in the stock Crypt.
- **A refused class exit** says stock's "You may not go through this exit!".
- **The answers that depend on class** (the preacher, the Dark Robed Figure, the spirits)
  come after the NPC's lead-in line, as stock quest texts do. A script's fallback line
  needs a `:` (stock reads no line without one), so it starts with an always-true
  `maxlevel 9999`.
- **Fork abilities:** `OffhandOnly` 1018, `CompanionAccuracy` 1019, `CompanionDamage` 1020,
  `ClassWeaponGiven` 1021 (hidden). No catch ability was needed.
- **The content overlay** gained: typed exits (any stock type, with its four parameters),
  room changes that set a stock room's NPC and script, training-shop fields, monsters'
  carried items, items' accuracy, strength and class list, messages (from 20000), and text
  blocks longer than one record (stored in parts). A new room now holds **no random
  monsters unless the content says so**: it has no monster group of its own, and a copied
  limit with no group stalled the server as a player walked in.

- [Summary](#summary)
- [How stock does it](#how-stock-does-it)
- [Level 1-10 trainers](#level-1-10-trainers)
  - [The Dervish: the Halls of Training](#the-dervish-the-halls-of-training)
  - [The Avenger: a secret room in the Temple](#the-avenger-a-secret-room-in-the-temple)
  - [The War Mage: above the Magic Shoppe](#the-war-mage-above-the-magic-shoppe)
  - [The Assassin: the ink shop](#the-assassin-the-ink-shop)
  - [The Warden: the Silvermere stables](#the-warden-the-silvermere-stables)
- [The level-10 tomb quests](#the-level-10-tomb-quests)
- [Class weapons](#class-weapons)
- [Fitting it into silvermere](#fitting-it-into-silvermere)
- [Numbering](#numbering)
- [Decisions](#decisions)
- [Open questions](#open-questions)
- [Brainstorm](#brainstorm)

## Summary

Each of the fork's five classes (War Mage 16, Warden 17, Avenger 18, Assassin 19, Dervish 20)
gets what a stock class has:

- **A level 1-10 trainer of its own,** each in a place that suits the class. Until now
  they train at the universal trainers.
- **A level-10 quest in the Crypt below the graveyard,** ending in the class's tomb, where
  its Super trainer trains level 11 (the only place a stock class can). Two new wings
  branch off the Crypt's Shadowed Hall: the fighters' (War Mage, Warden, Dervish) off
  1/1420 and the stealth casters' (Avenger, Assassin) off 1/1421.
- **A class weapon,** given once by the class's spirit in its tomb. The five weapons are in
  [Class weapons](#class-weapons).

The Warden's trainer comes with a second, bigger stable in Silvermere that sells leashes for
gold, including a stronger second tier of companions.

## How stock does it

- **Levels 1-10:** each class has a Training Room (a training shop for its class, levels
  1-10) off the Adventurer's Guild's Halls of Training (1/494-503); the Cleric's and the
  Priest's are in the Temple (1/521).
- **Level 11:** only a class's **Super trainer** trains it, and it waits in the class's
  **Large Tomb** under the graveyard. The way there is the level-10 quest:
  1. A **mummy** in the Crypt drops the **bone key** that opens the Sealed Tomb (1/1309).
  2. The **Shadowed Hall** (1/1377, 1419-1423) has a **class exit** for each class (exit type
     13: only that class may pass) into a marble passage.
  3. The passage ends at a **dead end with a glyph**: `push button` (a text exit) swings the
     wall round into a **Large Chamber** with a **guardian** (troll, fire lizard, ogre),
     who drops the key to the chamber's **iron door**.
  4. Beyond the door, a **Marble Chamber** hub, and off it **class-gated passages** to each
     class's **Large Tomb**: the class's **spirit** (an NPC), the Super trainer (training
     shop, level 11 only) and a **portal** home (`go portal`, back to 1/834).
- **Class weapons:** stock has almost none; a few weapons are limited to one class or two
  (Mage, Druid, Mage-or-Mystic). The quest gives no weapon.

## Level 1-10 trainers

Each is a stock training shop for its class (training, levels 1-10, `trainclass` the class),
copied from a stock class Training Room. The rooms are new (fork room numbers), joined to
stock rooms by new exits.

### The Dervish: the Halls of Training

- **The hall:** a one-room extension of the Halls of Training, **west of 1/491**
  (west is free there; west of 1/493 is stock's `push button` puzzle). Its name and
  description are copied from the Halls of Training (1/494).
- **The Dervish Training Room,** north of that hall room. Proposed description:

  > Sand crunches underfoot in this long, bare room, brought in by the sackful and raked
  > smooth each morning. Pairs of practice blades hang on pegs along the walls, and a ring
  > is painted on the floor, scuffed by years of turning feet. A lean, sun-darkened woman
  > watches from the edge of the ring, two curved knives at her belt. She is plainly the
  > trainer here.

### The Avenger: a secret room in the Temple

- **The secret room,** north of the **Temple Foyer** (1/2316), behind a **hidden exit**
  that nothing shows until it's opened.
- **Opening it:** in the foyer, `push catch` and then `pull lever`, **in that order**. The
  lever then opens the hidden door for a while, and **anyone** can go through while it's
  open (as stock's `push button` opens the Halls of Training's secret passage).
  - `push catch`: "You feel a small catch give under your fingers with a click." (Marks the
    character, a hidden fork ability.)
  - `pull lever` after the catch: the door opens. "Something clunks behind the coat racks,
    and a section of the north wall swings inward." The room sees it too. (Clears the mark.)
  - `pull lever` without the catch: "The lever won't budge."
- **The shifty preacher,** a new NPC in the foyer:
  - Looks: a thin man in a threadbare cassock with ink-stained fingers and a collection
    plate he never lets go of. His eyes keep drifting to the coat racks.
  - `ask preacher forgiveness`: "'Forgiveness, friend? The gods are generous, but their
    servants must eat. Forgiveness can be paid for in gold...'" **To an Avenger only**, he
    goes on: "'...or in blood,' he adds softly, watching your face."
  - `ask preacher blood`: **to an Avenger,** he leans in: "'The temple keeps its blades out
    of sight. Behind the coat racks there's a catch. Push it, then pull the lever beside
    it, and go north.'" **To anyone else:** "'Blood? I said nothing about blood.' He
    rattles his collection plate at you."
- **The Avenger Training Room,** with an **Avenger Trainer** NPC. Proposed description:

  > A narrow chapel without windows, lit by a single lamp of red glass. There is no altar
  > here, only a long rack of maces and flails, a scarred practice post and a ledger of
  > names, many of them crossed out. A hooded woman in dark leather sits by the ledger,
  > turning a mace slowly in her hands.

### The War Mage: above the Magic Shoppe

- **The War Mage Training Room,** **up** from the Magic Shoppe (1/398). Nothing hidden: go
  in, go up. Proposed description:

  > The upper floor of the Magic Shoppe has been cleared for something louder than
  > enchanting. Scorch marks blacken the floorboards around a battered armour stand, and
  > the walls are hung with notched blades, each with a spell-glyph etched near the hilt.
  > A broad-shouldered veteran of the Battle-Mage corps runs the room, a longsword on her
  > back and chalk on her fingers.

### The Assassin: the ink shop

- **The ink shop,** **north of Mystic Alley (1/396)**, one room. It is a training shop, but
  `LIST` gets no wares (see [Open questions](#open-questions)).
- **The Dark Robed Figure,** an NPC behind the counter:
  - `ask figure <anything>`, and greeting: "'Inks,' the figure murmurs. 'I make inks for
    Aiken, in the Magic Shoppe. Nothing else.'"
  - **To an Assassin:** "The figure glances at the door, then at the window, and lowers its
    voice. 'Ah. You're not here for ink. I can see to your training.'"
- Proposed description:

  > A cramped shop that smells of soot, oak galls and something sharper underneath. Jars of
  > ink in every colour crowd the shelves, and a little furnace glows in the corner beside
  > a rack of very thin, very clean knives. A figure in a dark robe works at the counter,
  > face hidden, grinding something in a mortar.

### The Warden: the Silvermere stables

Three rooms in a row **east of Park Street (1/165)**, like the Newhaven stable, with new NPCs:

1. **The yard** (east of 1/165), with an **intro NPC** who describes leashes (as Old
   Bannon and Deel do): what each kind of companion is good for, that a leash must be
   USEd in a Warden Stables, and that these ones cost gold. Proposed: **Corra Hale**, a
   brisk woman with a whistle round her neck.
2. **The stables** (east of the yard): the pens, a **stablemaster NPC** (proposed:
   **Tobias Wren**), and a **shop** that sells leashes **for gold**:
   - **Tier 1:** the four Newhaven companions, at their "real" prices: **24-50 gold**.
   - **Tier 2:** stronger companions, **100-500 gold**, each a few points better than its
     tier-1 cousin, some with **Magic 1** (as MajorMUD prices a few points of improvement
     at hundreds of gold).
   - **USE a leash here** bonds its companion. A Warden with a companion **gives it up**:
     the new one replaces it and **starts unnamed**.
3. **The Warden Training Room** (east of the stables), with a **Warden Trainer** NPC.
   Training up to level 10.

**The leashes (proposed numbers):**

| Tier | Companion | Price | Blows | Damage (base; per 100 levels) | Accuracy | Poison | Magic |
|---|---|---|---|---|---|---|---|
| 1 | hunting hound | 30 gold | 1 a round | 2-8; +50/+200 | 0 | | 0 |
| 1 | hawkling | 24 gold | 3 a round | 1-4; +15/+60 | +5 | | 0 |
| 1 | trained rattlesnake | 36 gold | 1 a round | 1-4; +28/+100 | 0 | 5 | 0 |
| 1 | drop bear | 50 gold | every other round | 3-16; +100/+400 | −5 | | 0 |
| 2 | war hound | 150 gold | 1 a round | 3-9; +55/+210 | +5 | | 0 |
| 2 | gyrfalcon | 250 gold | 3 a round | 1-5; +16/+65 | +8 | | 1 |
| 2 | desert viper | 200 gold | 1 a round | 1-5; +30/+105 | +2 | 8 | 0 |
| 2 | cave bear | 350 gold | every other round | 4-17; +105/+420 | −3 | | 0 |
| 2 | shadow mastiff | 500 gold | 1 a round | 3-10; +58/+220 | +8 | | 1 |

**Companion magic** (decided): a companion has a **Magic** level, as a weapon does. Like a
weapon's, it decides whether its blows can hurt a monster that needs magic weapons. This
changes today's companions: until now they could hurt anything; a tier-1 companion (Magic 0)
now can't hurt such a monster.

## The level-10 tomb quests

Two new wings off the Crypt's Shadowed Hall, each built the stock way. The tomb quests copy
stock's rooms (marble passages, a glyph dead end, a Large Chamber, a Marble Chamber hub,
Large Tombs), so the wings look and play like the stock ones.

- **Entrances (decided: class-gated):** a stock class exit lets exactly one class through,
  so each class gets **its own class exit** off its Shadowed Hall room, all leading into its
  wing's first passage (as stock's Shadowed Hall does). Proposed directions:
  - **The fighters' wing,** off **1/1420**: War Mage **northwest**, Warden **northeast**,
    Dervish **down**.
  - **The stealth casters' wing,** off **1/1421**: Avenger **northwest**, Assassin
    **northeast**.
- **Each wing:**
  1. A **marble passage** to a **dead end with a glyph**: `push button` swings the wall
     round into the wing's **Large Chamber**.
  2. The chamber's **guardian** drops the **key** to its **iron door**. Proposed: a
     **bone colossus** for the fighters' wing (troll-like) and a **grave wraith** for the
     stealth casters' (fire lizard-like), each with its own key.
  3. Past the door, a **Marble Chamber** hub with a **class-gated passage** to each of the
     wing's tombs.
  4. Each **Large Tomb**: the class's **spirit** (War Mage spirit, Warden spirit, and so
     on), the class's **Super trainer** (level 11 only, as stock's) and the **portal** home
     (`go portal`, to 1/834).
- **The spirit's reward:** asked about it (for example `ask spirit weapon`), the spirit gives
  its class's weapon **once** per character (a hidden fork ability marks it, as the
  Newhaven stable's one-starter rule does).

## Class weapons

Given once by each class's tomb spirit, each designed with you. What each had to fit:

| Class | Weapons it can use | Notes |
|---|---|---|
| War Mage | any (no shield) | used with Spellstrike |
| Warden | two-handed or ranged | the Warden's companion fights beside it |
| Avenger | blunt | backstabs with it (blunt backstab) |
| Assassin | sharp | backstabs; mostly opens with spells |
| Dervish | one-handed, two at once | a pair? one for each hand? |

Each will be a class item (stock's "class item" ability limits it to its class) and a
level-10 quest reward in power. The measure is stock's **flametongue**: a one-handed sword,
4-18 at speed 1400, +6 accuracy, from level 10, Magical 2, +2 spellcasting, faster mana,
light, and a small free fire spell on its hits.

### The War Mage: the Runic Athame

> A ritual blade grown to the length of a greatsword, forged from mithril and covered in
> acid-etched runes that glow with an inner light.

- **Form (decided):** a two-handed sharp blade. A War Mage never carries a shield, so both
  hands are free for it.
- **Power (decided): it feeds Spellstrike.** While it is wielded, a spellstrike's cast takes
  less of the round. The weapon carries the War Mage's own (hidden) Spellstrike discounts,
  which add to the class's: Spellstrike already totals them from every source, so this needs
  no code.

| Field | Proposed | For comparison |
|---|---|---|
| damage | 7-24 | greatsword 6-21; flametongue 4-18 |
| speed | 1900 | greatsword 2200 |
| accuracy | +4 | flametongue +6 |
| from level | 10 | flametongue 10 |
| strength needed | 55 | greatsword 75 (mithril is light) |
| Magical | 2 | flametongue 2 |
| light | a little (the glowing runes) | flametongue gives light |
| **Spellstrike, success** | **+10%** off a successful spellstrike's blow cost | the class's own 10%: 20% in all |
| **Spellstrike, failure** | **+10%** off a failed one's | the class's 30% + 40%: 80% in all |
| class item | War Mage only | |

With it, a successful spellstrike costs 80% of a blow (90% without it), and a failed one
20% (30% without it).

### The Dervish: the Blood-Jeweled Kris

> A short, wavy-bladed kris, its hilt set with dark red jewels that seem to drink the
> light. It is balanced for the off-hand and nothing else.

- **One blade (decided):** the spirit gives a single kris. The Dervish wields its own main
  weapon; the kris is for the other hand.
- **Off-hand only (decided):** it can only be held with OFFHAND. WIELD refuses it ("The kris
  is balanced for the off-hand."). That needs a fork item ability, **`OffhandOnly`**, and
  two small code changes: WIELD (and EQUIP) refuse an item with it, and when the main hand
  empties, an off-hand-only weapon **stays in the off-hand** rather than moving across (the
  main hand stays empty until the Dervish wields something).
- **Power (decided): it's quick.** A fast blade, so the off-hand gets more of its blows each
  round before the cap of 8.

| Field | Proposed | For comparison |
|---|---|---|
| damage | 4-13 | dagger 1-5; flametongue 4-18 |
| speed | 800 | dagger 900; flametongue 1400 |
| accuracy | +6 | flametongue +6; offsets the off-hand's −10 |
| from level | 10 | |
| Magical | 2 | flametongue 2 |
| class item | Dervish only | |
| off-hand only | yes | |

At level 10 with Agility 50 a speed-800 kris costs about 210 energy a blow: four or five
blows a round from the off-hand alone, so with a main-hand weapon the Dervish reaches the
cap of 8 about as early as it can.

### The Warden: the Darkwood Arbalest

> A heavy crossbow of near-black darkwood, its stock carved with running hounds and
> stooping hawks. Its string hums faintly whenever an animal is near.

- **Form (decided):** a heavy crossbow: two-handed, marked **Ranged**. Ranged combat is
  still only a marker, so for now it fights as any weapon does, and it becomes a true
  crossbow whenever ranged combat is built.
- **Power (decided): it strengthens the companion.** While the Warden wields it, its
  companion's blows are surer and harder. Two fork item abilities, read by the companion's
  blow (a small code change in `combat/pet.rs`, which already builds the blow from its
  owner's side):
  - **`CompanionAccuracy`**: added to the companion's attack value.
  - **`CompanionDamage`**: the companion's damage raised by this percentage.

| Field | Proposed | For comparison |
|---|---|---|
| damage | 8-25 | stock heavy crossbow 7-22 |
| speed | 2000 | stock heavy crossbow 2025 |
| accuracy | +8 | stock heavy crossbow +5 |
| from level | 10 | |
| strength needed | 65 | stock heavy crossbow 70 |
| Magical | 2 | flametongue 2 |
| ranged | yes | |
| **companion accuracy** | **+10** | the hawkling's own +5 |
| **companion damage** | **+25%** | |
| class item | Warden only | |

### The Avenger: the censer of judgement

> A heavy iron censer on a length of blackened chain, swung like a flail. Embers smoulder
> inside it, and wherever it goes it leaves a thin trail of bitter incense, like a church
> after a funeral.

- **Form (decided):** a flail: one-handed and blunt, so the Avenger can backstab with it
  (blunt backstab).
- **Power (decided): surer spell backstabs.** While it is wielded, the spell backstab's
  stealth roll loses only 10 for the casting, not 20. The censer carries the
  `SpellBackstabStealth` ability at −10, which adds to the class's 20, so this needs no code.

| Field | Proposed | For comparison |
|---|---|---|
| damage | 5-17 | platinum mace 6-15; black flail 9-20 |
| speed | 1600 | platinum mace 1200; black flail 1700 |
| accuracy | +4 | platinum mace +12; black flail +6 |
| from level | 10 | |
| strength needed | 55 | black flail 70 |
| Magical | 2 | flametongue 2 |
| light | a little (the embers) | |
| **spell backstab stealth** | **−10 to the penalty** (−20 becomes −10) | |
| class item | Avenger only | |

### The Assassin: the hexknife

> A thin stiletto of blued steel, its needle blade scribed from hilt to point with tiny,
> cramped sigils in a hand no one admits to teaching. The letters are too small to read,
> and they feel warm.

- **Form (decided):** a stiletto: one-handed and sharp, made for the killing thrust. Like
  stock's daggers it carries "Alter Backstab Attack Value", so it backstabs.
- **Power (decided): harder spell backstabs.** While it is wielded, a spell backstab raises
  the spell's roll by 150%, not 100% (the roll counts two and a half times, not twice,
  before the level, stealth and Backstab bonuses). The hexknife carries `SpellBackstab` at
  +50, which adds to the class's 100, so this needs no code. The Avenger's censer makes the
  backstab land more often; the hexknife makes it land harder.

| Field | Proposed | For comparison |
|---|---|---|
| damage | 3-12 | stiletto 1-4; curved black dagger 2-10; flametongue 4-18 |
| speed | 900 | stiletto 800; dagger 900 |
| accuracy | +6 | flametongue +6 |
| from level | 10 | |
| strength needed | 40 | |
| Magical | 2 | flametongue 2 |
| backstab attack value | +10 | stock's dagger +10 |
| **spell backstab** | **+50%** (+100% becomes +150%) | |
| class item | Assassin only | |

At level 50 (stealth 160), a spell backstab rolling 70-100 comes out about 13-16% stronger
with it than without: the raise counts only on the spell's roll, not on the level and
stealth parts.

## Fitting it into silvermere

All of it is content (`content/world.toml`, `content/pets.toml`) except where noted. The
content overlay needs a few new kinds of record:

| Needed | Where |
|---|---|
| **Exit types** on new rooms and room changes: class exits, key exits, hidden exits, remote actions, text exits (`push button`, `go portal`) | content model: an exit becomes either a room number or a table with its type and parameters |
| **A room change that sets a stock room's NPC and script** (the preacher and the foyer's `push catch` / `pull lever` lines) | room changes gain `specmon` and `script` |
| **Training shops:** `trainclass`, lowest and highest level | shops gain those fields (copied from a stock training shop) |
| **Monsters that drop a key** | monsters gain carried items |
| **Tier-2 companions with Magic** | `content/pets.toml` gains `magic`; the companion's blow checks it against the monster (code, `combat/pet.rs`) |
| **Rebonding:** a new leash replaces the companion, unnamed | the stable script sets `PetKind`; the code clears the name when the kind changes (code) |
| **Two hidden fork abilities:** the pushed catch, and the class weapon given | `silvermere-types` `fork` |
| **A weapon that strengthens the companion** (the Warden's arbalest) | fork item abilities `CompanionAccuracy`, `CompanionDamage`; `combat/pet.rs` |
| **An off-hand-only weapon** (the Dervish's kris): WIELD refuses it, and it doesn't move to an emptied main hand | fork item ability `OffhandOnly`; `items/equip.rs`, `combat/dual.rs` |
| **Class-conditional answers** (the preacher, the Dark Robed Figure) | no code: ASK runs the script its answer links to, and scripts have a `class` clause |

## Numbering

Proposed, in the fork's ranges:

| What | Numbers |
|---|---|
| Rooms: Warden stables (yard, stables, trainer) | 1/9010-9012 |
| Room: War Mage Training Room | 1/9020 |
| Room: the ink shop | 1/9030 |
| Room: Avenger Training Room | 1/9040 |
| Rooms: Dervish hall, Dervish Training Room | 1/9050-9051 |
| Rooms: the fighters' wing | 1/9100-9119 |
| Rooms: the stealth casters' wing | 1/9120-9139 |
| Shops: five training rooms, five Super trainers, the Silvermere stable | 902-912 |
| Monsters: the NPCs, the guardians, the spirits | 9002-9020 |
| Items: the leashes (tier 1 and 2), the two keys, the class weapons | 9010-9039 |
| Companion kinds: tier 2 | 5-9 |
| Text blocks | 20100-20299 |

## Decisions

1. Each fork class gets a level 1-10 trainer of its own: the Dervish west of 1/491 then
   north; the Avenger behind the Temple Foyer's `push catch` / `pull lever`; the War Mage
   up from the Magic Shoppe; the Assassin north of 1/396 with the Dark Robed Figure; the
   Warden east of Park Street with a three-room stable.
2. The shifty preacher mentions blood only to an Avenger, and tells only an Avenger how to
   get in. Once the lever is pulled, anyone can enter.
3. The Silvermere stable sells tier-1 leashes for 24-50 gold and tier-2 for 100-500 gold.
   A new leash replaces the companion, which starts unnamed.
4. A companion's Magic gates what it can hurt, as a weapon's does.
5. Two tomb wings off the Shadowed Hall, built the stock way: fighters off 1/1420, stealth
   casters off 1/1421, each class with its own class exit into its wing.
6. Each class's tomb spirit gives its class weapon once.
7. The War Mage's weapon is the **Runic Athame**, a two-handed mithril blade that feeds
   Spellstrike.
8. The Dervish's weapon is the **Blood-Jeweled Kris**: one quick blade, off-hand only. WIELD
   refuses it, and it stays in the off-hand when the main hand empties.
9. The Warden's weapon is the **Darkwood Arbalest**: a heavy crossbow (Ranged) that makes its
   companion's blows surer and harder.
10. The Avenger's weapon is the **censer of judgement**: a flail whose wielder's spell
    backstabs are spotted less often (the stealth penalty −10, not −20).
11. The Assassin's weapon is the **hexknife**: a stiletto whose wielder's spell backstabs hit
    harder (+150%, not +100%).

## Open questions

1. **The class weapons:** each to be designed with you. The War Mage's (the Runic Athame) is
   designed, and so are the Dervish's (the Blood-Jeweled Kris) and the Warden's (the
   Darkwood Arbalest), the Avenger's (the censer of judgement) and the Assassin's (the
   hexknife): all five are designed. Their numbers are proposals.
2. **LIST in the ink shop:** a training shop may answer LIST with something; the ink shop
   should say nothing useful. To be checked when it's built, and silenced if it does.
3. **The proposed names, descriptions, guardians, leash numbers and directions** above.
4. **ASK with "anything" (the Dark Robed Figure):** a talk block matches keywords, so the
   figure answers a list of likely words (INK, INKS, TRAIN, TRAINING, AIKEN, SHOP, WORK...)
   and greets anyone who asks with no question.

## Brainstorm

- The Silvermere stable could buy back a released companion's leash for a little gold.
- The tomb spirits could say a few lines about their class's founders.
