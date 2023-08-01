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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

/// Weight functions needed for pallet_cosmos_accounts.
pub trait WeightInfo {
	fn connect() -> Weight;
}

/// Weights for pallet_cosmos_accounts using the Horizon node and recommended hardware.
pub struct HorizonWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for HorizonWeight<T> {
	fn connect() -> Weight {
		Weight::from_ref_time(50_000_000)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
		}
}
