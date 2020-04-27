use crate::prelude::*;
use crate::map::{Map, MapBuilder};
use crate::room::Room;
use crate::enemy::*;
use crate::task::UntypedPromise;
use crate::item::*;

use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub enum HealthModifyReason {
	Heal,
	Attack,
}


#[derive(Copy, Clone, Debug)]
pub enum GameCommand {
	GivePlayerItem(Item, usize),
	ConsumePlayerItem(Item, usize),
	ModifyPlayerHealth(i32, HealthModifyReason),
	StarvePlayer,
	SatePlayer,
	MovePlayer(Direction),

	AttackEnemy(i32),
}


#[derive(Debug)]
pub struct GameState {
	pub map: Map,
	pub player: Player,

	pub enemies: HashMap<Location, Enemy>,
}


impl GameState {
	pub fn new() -> GameState {
		GameState {
			map: Map::new(),
			player: Player::new(),

			enemies: HashMap::new(),
		}
	}

	pub fn try_move_player(&mut self, dir: Direction) -> bool {
		let room = self.map.get(self.player.location)
			.expect("Player somehow not in a room");

		if room.door(dir) {
			self.player.location = self.player.location.offset_in_direction(dir);
			MapBuilder::new(&mut self.map).generate_room_at(self.player.location);
			true
		} else {
			false
		}
	}

	pub fn remove_encounter_at(&mut self, loc: Location) {
		if let Some(room) = self.map.get(loc) {
			self.map.replace(loc, Room { encounter: None, .. room });
		}

		self.enemies.remove(&loc);
	}

	pub fn spawn_enemy_at(&mut self, loc: Location, boss: bool) {
		let archetype = EnemyArchetype::choose(boss);
		self.enemies.insert(loc, archetype.new());
	}

	pub fn get_enemy(&self, loc: Location) -> Option<Enemy> {
		self.enemies.get(&loc).copied()
	}

	pub fn update_enemy(&mut self, loc: Location, enemy: Enemy) {
		self.enemies.insert(loc, enemy);
	}

	pub fn submit_command(&mut self, event: GameCommand, promise: UntypedPromise) {
		match event {
			GameCommand::GivePlayerItem(item, n) => {
				self.player.inventory.add_n(item, n);
				promise.void().fulfill(());
			}

			GameCommand::ConsumePlayerItem(item, n) => {
				let success = self.player.inventory.take_n(item, n);
				promise.bool().fulfill(success);
			}

			GameCommand::ModifyPlayerHealth(n, _) => {
				self.player.health += n;
				promise.bool().fulfill(self.player.health > 0);
			}

			GameCommand::StarvePlayer => {
				self.player.hunger -= 1;
				promise.bool().fulfill(self.player.hunger > 0);
			}

			GameCommand::SatePlayer => {
				self.player.hunger = 10;
				promise.void().fulfill(());
			}

			GameCommand::MovePlayer(dir) => {
				promise.bool().fulfill(self.try_move_player(dir));
			}

			GameCommand::AttackEnemy(dmg) => {
				let loc = self.player.location;
				if let Some(mut enemy) = self.get_enemy(loc) {
					enemy.health -= dmg;
					self.update_enemy(loc, enemy);
				}

				promise.void().fulfill(());
			}
		}
	}
}



#[derive(Debug)]
pub struct Player {
	pub location: Location,
	pub health: i32,
	pub hunger: i32,

	pub inventory: Inventory,
}

impl Player {
	pub fn new() -> Self {
		let mut inventory = Inventory::new();
		inventory.add_n(Item::Food, 20);
		inventory.add_n(Item::Treasure, 5);

		Player {
			location: Location(0, 0),
			health: 15,
			hunger: 10,

			inventory,
		}
	}

	pub fn attack(&self) -> i32 {
		let weapon_stat: i32 = self.inventory.iter_equipment()
			.map(Equipment::attack_bonus).sum();

		2 + weapon_stat
	}
	pub fn defense(&self) -> i32 {
		self.inventory.iter_equipment().map(Equipment::defense_bonus).sum()
	}

	pub fn is_dead(&self) -> bool { self.health <= 0 }
}


#[derive(Debug)]
pub struct Inventory {
	items: Vec<Item>,
	treasure: i32,
	food: i32,
}

impl Inventory {
	pub fn new() -> Inventory {
		Inventory {
			items: Vec::new(),
			treasure: 0,
			food: 0,
		}
	}

	pub fn add(&mut self, item: Item) { self.add_n(item, 1) }

	pub fn add_n(&mut self, item: Item, n: usize) {
		match item {
			Item::Treasure => { self.treasure += n as i32 }
			Item::Food => { self.food += n as i32 }
			_ => {
				use std::iter;
				self.items.extend(iter::repeat(item).take(n))
			}
		}
	}

	pub fn take(&mut self, item: Item) -> bool { self.take_n(item, 1) }

	pub fn take_n(&mut self, item: Item, n: usize) -> bool {
		if self.count(item) < n {
			return false
		}

		match item {
			Item::Treasure => self.treasure -= n as i32,
			Item::Food => self.food -= n as i32,
			_ => for _ in 0..n {
				self.items.remove_item(&item);
			}
		}

		true
	}

	pub fn count(&self, item: Item) -> usize {
		match item {
			Item::Treasure => self.treasure as usize,
			Item::Food => self.food as usize,
			_ => self.items.iter()
				.filter(|&&i| i == item)
				.count()
		}
	}

	pub fn has(&self, item: Item) -> bool { self.count(item) > 0 }

	pub fn iter_equipment(&self) -> impl Iterator<Item=Equipment> + '_ {
		self.items.iter()
			.filter_map(|i| match i {
				Item::Equipment(e) => Some(*e),
				_ => None,
			})
	}
}