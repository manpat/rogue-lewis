
Dungeon
=======
- Randomly generated grid of rooms, connected by doors/corridors
    - Some doors can lead to ungenerated rooms, which will be generated upon entry
- Has one exit

Rooms
-----
- Must have at least one door
- Can contain one of the following:
    - Food                      ~33%
    - Encounters w/ monsters    ~13%
    - Treasure                  ~10%
    - Traps                     ~5%
    - Equipment                 ~4%
    - Merchant                  ~3.3%
    - Chests                    ~3%
    - Keys                      ~2.6%
    - Maps                      ~2%
    - Boss                      ~1%
- Can be an exit
    - Spawns a boss which must be beaten to escape

Player
======
- Has several stats
    - Health    starts at 15
    - Hunger    starts at 10
    - Damage    starts at 2
    - Shield    starts at 0
- Has an inventory, which can contain several of any item *except* maps, which the player can only hold one of
- Can equip equipment
- Starts with 20 food and 5 treasure in inventory

Equipment
---------
Have some effect on player stats that are applied once, immediately on equiping
- Sword:    damage +1
- Axe:      damage +2
- Shield:   shield +1
- Armour:   shield +2
- Potion:   health x2

Map
---
- Shown with "map" command
- Initially player can only see rooms they've visited
- Once player obtains map item, can see all rooms already generated
    - Note: this isn't necessarily all rooms, as some rooms can have doors to ungenerated rooms
- Shows which doors/corridors a room has, regardless of whether they lead to generated rooms
- Shows the contents of a room if it's been previously visited
    - e.g., if a room contains a monster or chest and the player leaves, it will be marked as such on the map
    - Rooms revealed by the map item do *not* show their contents until visited, but *will* show the exit


Encounters
==========

Chest
-----
- Require a Key to open
- Can contain [1, 5) random items *excluding* maps
- Uninteractible once opened


Food/Treasure/Key
-----------------
- Item added to inventory immediately

Equipment
---------
- Item immediately equipped

Map
---
- Added immediately to inventory *if* player doesn't already have one
- If player already has a map, the map remains in the room so it can be collected later
- Allows player to see the entirety of initially generated map with "map" command
    - Until this point the player can only see parts of the map they've visited

Monster
-------
- Has stats: health, and damage
- Is one of:
    - Goblin:   health: 3, damage: 1
    - Ogre:     health: 6, damage: 1
    - Orc:      health: 5, damage: 2
    - Gargoyle: health: 2, damage: 4
- Upon encounter, immediately enters a battle
- Remains in room until defeated

Boss
----
- Has stats: health, damage, and shield
- Is one of:
    - Guardian: health: 15, damage: 1, shield: 2
    - Minotaur: health: 9,  damage: 4, shield: 1
- Upon encounter, immediately enters a battle
- Remains in room until defeated

Trap
----
- Is one of:
    - Tripwire
        - Player trips and drops [1, 2] items
        - 1/3 chance to notice tripwire and avoid
        - Player is given option to attemt to disarm - see Trap Disarm Loop
    - Bolt
        - Fires a bolt at player, dealing [1, 2] damage
        - 1/3 chance to miss
        - Player is given option to attemt to disarm - see Trap Disarm Loop
    - Ambush
        - A monster spawns and a battle is entered
        - monster has 3/5 chance to deal it's damage before player has a chance to flee. This ignores any shield the player has
        - 1/5 chance for player to get automatic free strike on monster
            - BUG: monster immediately dies in this case
        - After first strikes, this is a regular battle
    - Portal


Regular Game Loop
=================



Battle Loop
===========
- Can always be fleed, but 40% chance of enemy attacking you
- Player can:
    - Fight
        - See below
    - Eat/Heal
        - Consume one food to recover [1, 4) health
        - if monster:
            - 6/10 chance of monster attacking but missing
            - 2/10 chance of monster attacking and hitting
            - 2/10 chance of monster attacking and criting
        - if boss:
            - 6/13 chance of monster attacking but missing
            - 4/13 chance of monster attacking and hitting
            - 3/13 chance of monster attacking and criting
        - enemy does not attack if no food available in inventory
    - Flee
        - 2/5 chance of enemy attacking. ignores shield stat 

- On each successful attack:
    - the attackers damage is subtracted from the attackees health
    - there's a 3/4 chance of the relevant party's shield stat being subtracted from attack's damage (clamped so that it does not go negative)
        - Note: the shield stat is ignored while fleeing

- On crit:
    - damage stat for relevant party is temporarily doubled

- When player chooses to fight
    - player rolls [1, 10]
    - enemy rolls [1, 10]  (if monster) or [1, 13]  (if boss)
    - if rolls are equal nothing happens
    - if player rolls higher and player rolls:
        - a 1:              nothing happens
        - in [9, 10]:       player crits
        - anything else:    player attacks regularly
    - BUG?: boss's shield stat is _always_ applied
        - when player crits *and* boss 'successfully' uses shield, player instead deals double damage despite message saying otherwise

    - if player rolls lower and enemy rolls:
        - if monster:
            - a 1:              nothing happens
            - in [9, 10]:       monster crits
            - anything else:    monster attacks regularly
        - if boss:
            - in [1, 3]:        nothing happens
            - in [11, 13]:      boss crits
            - anything else:    boss attacks regularly



Merchant Loop
=============


Trap Disarm Loop
================