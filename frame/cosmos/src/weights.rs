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

use frame_support::weights::Weight;
use sp_core::Get;
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cosmos.
pub trait WeightInfo {
	fn msg_send() -> Weight;
}

/// Weights for pallet_cosmos using the Horizon node and recommended hardware.
pub struct HorizonWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for HorizonWeight<T> {
	fn msg_send() -> Weight {
		use pallet_balances::WeightInfo;
		pallet_balances::weights::SubstrateWeight::<T>::transfer()
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
