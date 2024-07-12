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

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use pallet_balances::WeightInfo;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{coin::Coin, tx::Any};
use pallet_cosmos_x::msgs::{MsgHandlerError, MsgHandlerErrorInfo};
use pallet_cosmos_x_bank_types::MsgSend;
use sp_runtime::{format_runtime_string, SaturatedConversion};

pub struct MsgSendHandler<T>(PhantomData<T>);

impl<T> Default for MsgSendHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_x::msgs::MsgHandler for MsgSendHandler<T>
where
	T: pallet_cosmos::Config,
{
	fn handle(&self, msg: &Any) -> Result<Weight, MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		let MsgSend { from_address, to_address, amount } = Self::to_msg(msg)
			.map_err(|e| MsgHandlerErrorInfo { weight: total_weight, error: e })?;

		let from = T::AddressMapping::into_account_id(from_address.address);
		let to = T::AddressMapping::into_account_id(to_address.address);
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

		let weight = Self::send_coins(from, to, amount)?;
		total_weight = total_weight.saturating_add(weight);

		Ok(total_weight)
	}
}

impl<T> MsgSendHandler<T>
where
	T: pallet_cosmos::Config,
{
	fn to_msg(msg: &Any) -> Result<MsgSend, MsgHandlerError> {
		hp_io::cosmos::protobuf_to_scale(msg)
			.map(|value| Decode::decode(&mut &value[..]))
			.ok_or(MsgHandlerError::InvalidMsg)?
			.map_err(|_| MsgHandlerError::InvalidMsg)
	}

	fn send_coins(
		from: T::AccountId,
		to: T::AccountId,
		amount: sp_std::vec::Vec<Coin>,
	) -> Result<Weight, MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		for amt in amount.iter() {
			if T::NativeDenom::get() == amt.denom {
				T::Currency::transfer(
					&from,
					&to,
					amt.amount.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: MsgHandlerError::Custom(format_runtime_string!("Failed to transfer")),
				})?;

				total_weight = total_weight.saturating_add(
					pallet_balances::weights::SubstrateWeight::<T>::transfer_allow_death(),
				);
			} else {
				// TODO: Asset support planned
				return Err(MsgHandlerErrorInfo {
					weight: total_weight,
					error: MsgHandlerError::Unsupported,
				});
			}
		}

		Ok(total_weight)
	}
}
