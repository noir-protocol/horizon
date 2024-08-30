// This file is part of Horizon.

// Copyright (C) 2023 Haderech Pte. Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use pallet_cosmos_types::{
	events::{self, CosmosEvents, EventManager as _},
	store::{self, BasicGasMeter, Gas, GasMeter},
};

#[derive(Clone, Debug, Default)]
pub struct EventManager {
	events: CosmosEvents,
}

impl events::EventManager for EventManager {
	fn new() -> Self {
		Self::default()
	}

	fn events(&self) -> CosmosEvents {
		self.events.clone()
	}

	fn emit_event(&mut self, event: events::CosmosEvent) {
		self.events.push(event);
	}

	fn emit_events(&mut self, events: CosmosEvents) {
		self.events.extend(events);
	}
}

pub struct Context {
	pub gas_meter: BasicGasMeter,
	pub event_manager: EventManager,
}

impl store::Context for Context {
	type GasMeter = BasicGasMeter;
	type EventManager = EventManager;

	fn new(limit: Gas) -> Self {
		Self { gas_meter: Self::GasMeter::new(limit), event_manager: Self::EventManager::new() }
	}

	fn gas_meter(&mut self) -> &mut Self::GasMeter {
		&mut self.gas_meter
	}

	fn event_manager(&mut self) -> &mut Self::EventManager {
		&mut self.event_manager
	}
}
