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
use pallet_cosmos_types::{
	coin::Coin,
	events::{EventAttribute, ATTRIBUTE_KEY_AMOUNT, ATTRIBUTE_KEY_SENDER},
	msgservice::{MsgHandlerError, MsgHandlerErrorInfo},
	traits::ToStringBytes,
	tx::{AccountId, Any},
};
use pallet_cosmos_x_bank_types::{
	events::{ATTRIBUTE_KEY_RECIPIENT, EVENT_TYPE_TRANSFER},
	tx::MsgSend,
};
use sp_runtime::{format_runtime_string, SaturatedConversion};

pub struct MsgSendHandler<T>(PhantomData<T>);

impl<T> Default for MsgSendHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgSendHandler<T>
where
	T: pallet_cosmos::Config,
{
	fn handle(&self, msg: &Any) -> Result<Weight, MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		let MsgSend { from_address, to_address, amount } = Self::to_msg(msg)
			.map_err(|e| MsgHandlerErrorInfo { weight: total_weight, error: e })?;

		match Self::send_coins(from_address, to_address, amount) {
			Ok(weight) => {
				total_weight = total_weight.saturating_add(weight);
			},
			Err(e) => {
				return Err(MsgHandlerErrorInfo {
					weight: total_weight.saturating_add(e.weight),
					error: e.error,
				});
			},
		};

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
		from: AccountId,
		to: AccountId,
		amount: sp_std::vec::Vec<Coin>,
	) -> Result<Weight, MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		let from_acc = T::AddressMapping::into_account_id(from.address);
		let to_acc = T::AddressMapping::into_account_id(to.address);
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

		for amt in amount.iter() {
			if T::NativeDenom::get() == amt.denom {
				T::Currency::transfer(
					&from_acc,
					&to_acc,
					amt.amount.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: MsgHandlerError::Custom(format_runtime_string!("Failed to transfer")),
				})?;

				total_weight = total_weight.saturating_add(
					pallet_balances::weights::SubstrateWeight::<T>::transfer_keep_alive(),
				);
			} else {
				// TODO: Asset support planned
				return Err(MsgHandlerErrorInfo {
					weight: total_weight,
					error: MsgHandlerError::Unsupported,
				});
			}
		}

		pallet_cosmos::Pallet::<T>::deposit_event(pallet_cosmos::Event::Executed(
			pallet_cosmos_types::events::Event {
				r#type: EVENT_TYPE_TRANSFER.into(),
				attributes: sp_std::vec![
					EventAttribute { key: ATTRIBUTE_KEY_SENDER.into(), value: from.bech32 },
					EventAttribute { key: ATTRIBUTE_KEY_RECIPIENT.into(), value: to.bech32 },
					EventAttribute {
						key: ATTRIBUTE_KEY_AMOUNT.into(),
						value: amount.to_bytes().map_err(|_| MsgHandlerErrorInfo {
							weight: total_weight,
							error: MsgHandlerError::InvalidMsg
						})?
					},
				],
			},
		));

		Ok(total_weight)
	}
}
