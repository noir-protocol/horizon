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

use alloc::{string::String, vec::Vec};
use cosmos_sdk_proto::{
	cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin},
	traits::Message,
	Any,
};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement},
};
use pallet_assets::WeightInfo as _;
use pallet_balances::WeightInfo as _;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{
	address::address_from_bech32,
	coin::amount_to_string,
	errors::RootError,
	events::{CosmosEvent, EventAttribute, ATTRIBUTE_KEY_AMOUNT, ATTRIBUTE_KEY_SENDER},
	msgservice::MsgHandlerErrorInfo,
};
use pallet_cosmos_x_bank_types::events::{ATTRIBUTE_KEY_RECIPIENT, EVENT_TYPE_TRANSFER};
use sp_runtime::{traits::Convert, SaturatedConversion};

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
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		let MsgSend { from_address, to_address, amount } = MsgSend::decode(&mut &*msg.value)
			.map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::UnpackAnyError.into(),
			})?;

		match Self::send_coins(from_address, to_address, amount) {
			Ok((weight, msg_events)) => {
				total_weight = total_weight.saturating_add(weight);
				Ok((total_weight, msg_events))
			},
			Err(e) => Err(MsgHandlerErrorInfo {
				weight: total_weight.saturating_add(e.weight),
				error: e.error,
			}),
		}
	}
}

impl<T> MsgSendHandler<T>
where
	T: pallet_cosmos::Config,
{
	fn send_coins(
		from_address: String,
		to_address: String,
		amount: Vec<Coin>,
	) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let mut total_weight = Weight::zero();

		let from_addr = address_from_bech32(&from_address).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: RootError::InvalidAddress.into(),
		})?;

		let to_addr = address_from_bech32(&to_address).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: RootError::InvalidAddress.into(),
		})?;

		let from_account = T::AddressMapping::into_account_id(from_addr);
		let to_account = T::AddressMapping::into_account_id(to_addr);
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

		for amt in amount.iter() {
			let transfer_amount = amt.amount.parse::<u128>().map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::InvalidCoins.into(),
			})?;

			if T::NativeDenom::get() == amt.denom {
				T::NativeAsset::transfer(
					&from_account,
					&to_account,
					transfer_amount.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: RootError::InsufficientFunds.into(),
				})?;

				total_weight = total_weight.saturating_add(
					pallet_balances::weights::SubstrateWeight::<T>::transfer_keep_alive(),
				);
			} else {
				let asset_id = T::AssetToDenom::convert(amt.denom.clone()).map_err(|_| {
					MsgHandlerErrorInfo {
						weight: total_weight,
						error: RootError::InvalidCoins.into(),
					}
				})?;
				T::Assets::transfer(
					asset_id,
					&from_account,
					&to_account,
					transfer_amount.saturated_into(),
					Preservation::Preserve,
				)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: RootError::InsufficientFunds.into(),
				})?;
				total_weight = total_weight.saturating_add(
					pallet_assets::weights::SubstrateWeight::<T>::transfer_keep_alive(),
				);
			}
		}

		let msg_event = pallet_cosmos_types::events::CosmosEvent {
			r#type: EVENT_TYPE_TRANSFER.into(),
			attributes: alloc::vec![
				EventAttribute { key: ATTRIBUTE_KEY_SENDER.into(), value: from_address.into() },
				EventAttribute { key: ATTRIBUTE_KEY_RECIPIENT.into(), value: to_address.into() },
				EventAttribute {
					key: ATTRIBUTE_KEY_AMOUNT.into(),
					value: amount_to_string(&amount).into()
				},
			],
		};

		Ok((total_weight, alloc::vec![msg_event]))
	}
}
