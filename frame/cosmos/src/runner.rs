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

use crate::Config;
use frame_support::weights::Weight;
use sp_core::H160;

#[derive(Debug)]
pub struct RunnerError<E: Into<sp_runtime::DispatchError>> {
	pub error: E,
	pub weight: frame_support::weights::Weight,
}

pub trait Runner<T: Config> {
	type Error: Into<sp_runtime::DispatchError>;

	fn msg_send(
		from_address: &H160,
		to_address: &H160,
		amount: u128,
	) -> Result<Weight, RunnerError<Self::Error>>;
}
