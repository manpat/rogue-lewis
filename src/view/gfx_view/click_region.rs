use crate::prelude::*;


pub enum ClickRegionContext {
	Screen,
	Ground,
}

pub trait ClickRegion {
	type ClickResponse;

	fn context(&self) -> ClickRegionContext;
	fn position(&self) -> Vec2;
	fn size(&self) -> Vec2;

	fn while_mouse_over(&mut self) {}
	fn while_mouse_not_over(&mut self) {}
	fn on_click(&mut self) -> Self::ClickResponse;
}


pub enum ClickRegionEventType {
	Move, Click
}

pub struct ClickRegionEvent {
	ui_pos: Vec2,
	world_pos: Vec3,

	event_ty: ClickRegionEventType,
}


impl ClickRegionEvent {
	pub fn new_move(ui_pos: Vec2, world_pos: Vec3) -> Self {
		Self {
			ui_pos, world_pos,
			event_ty: ClickRegionEventType::Move
		}
	}

	pub fn new_click(ui_pos: Vec2, world_pos: Vec3) -> Self {
		Self {
			ui_pos, world_pos,
			event_ty: ClickRegionEventType::Click
		}
	}

	pub fn update<R>(&self, region: &mut R) -> Option<R::ClickResponse>
		where R : ClickRegion 
	{
		match self.event_ty {
			ClickRegionEventType::Move => {
				if self.region_contains_event(region) {
					region.while_mouse_over();
				} else {
					region.while_mouse_not_over();
				}
			}

			ClickRegionEventType::Click => {
				if self.region_contains_event(region) {
					return Some(region.on_click())
				}
			}
		}

		None
	}

	pub fn update_multiple<'r, R, I>(&self, regions: I) -> Option<R::ClickResponse>
		where I: IntoIterator<Item=&'r mut R>, R: ClickRegion + 'r
	{
		for r in regions {
			if let Some(response) = self.update(r) {
				return Some(response)
			} 
		}

		None
	}

	fn region_contains_event<R: ClickRegion>(&self, region: &R) -> bool {
		let event_pos = match region.context() {
			ClickRegionContext::Screen => self.ui_pos,
			ClickRegionContext::Ground => self.world_pos.to_xz(),
		};

		let diff = region.position() - event_pos;
		let extent = region.size() / 2.0;

		diff.x.abs() < extent.x && diff.y.abs() < extent.y 
	}
}