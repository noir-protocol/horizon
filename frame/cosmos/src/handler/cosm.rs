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

use crate::{
	errors::{CosmosError, CosmosErrorCode},
	handler::MsgHandler as MsgHandlerT,
	pallet,
	weights::WeightInfo,
	AddressMapping, BalanceOf, Config,
};
use core::marker::PhantomData;
use frame_support::{
	traits::{Currency, ExistenceRequirement},
	weights::Weight,
};
use sp_core::{Get, H160};

pub struct MsgHandler<T: Config> {
	_marker: PhantomData<T>,
}

impl<T: Config> MsgHandlerT<T> for MsgHandler<T>
where
	BalanceOf<T>: TryFrom<u128> + Into<u128>,
{
	fn msg_send(
		from_address: &H160,
		to_address: &H160,
		amount: u128,
	) -> Result<Weight, CosmosError> {
		let source = T::AddressMapping::into_account_id(*from_address);
		let target = T::AddressMapping::into_account_id(*to_address);
		let amount = amount.try_into().map_err(|_| CosmosError {
			weight: T::DbWeight::get().reads(2u64),
			error: CosmosErrorCode::ErrInvalidType,
		})?;
		T::Currency::transfer(&source, &target, amount, ExistenceRequirement::AllowDeath).map_err(
			|_| CosmosError {
				weight: T::DbWeight::get().reads(2u64),
				error: CosmosErrorCode::ErrInsufficientFunds,
			},
		)?;
		Ok(<T as pallet::Config>::WeightInfo::msg_send())
	}
}
