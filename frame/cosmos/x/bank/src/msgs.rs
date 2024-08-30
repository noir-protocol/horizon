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

use alloc::vec;
use cosmos_sdk_proto::{cosmos::bank::v1beta1::MsgSend, traits::Message, Any};
use frame_support::{
	pallet_prelude::*,
	traits::{fungibles::Mutate, tokens::Preservation, Currency, ExistenceRequirement},
};
use pallet_assets::WeightInfo as _;
use pallet_balances::WeightInfo as _;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{
	coin::amount_to_string,
	errors::{CosmosError, RootError},
	events::{EventAttribute, EventManager, ATTRIBUTE_KEY_AMOUNT, ATTRIBUTE_KEY_SENDER},
	store::{self, GasMeter},
};
use pallet_cosmos_x_bank_types::events::{ATTRIBUTE_KEY_RECIPIENT, EVENT_TYPE_TRANSFER};
use sp_runtime::{traits::Convert, SaturatedConversion};

pub struct MsgSendHandler<T>(PhantomData<T>);

impl<T> Default for MsgSendHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T, Context> pallet_cosmos_types::msgservice::MsgHandler<Context> for MsgSendHandler<T>
where
	T: pallet_cosmos::Config,
	Context: store::Context,
{
	fn handle(&self, msg: &Any, ctx: &mut Context) -> Result<(), CosmosError> {
		let MsgSend { from_address, to_address, amount } =
			MsgSend::decode(&mut &*msg.value).map_err(|_| RootError::UnpackAnyError)?;

		let from_account =
			T::AddressMapping::from_bech32(&from_address).ok_or(RootError::InvalidAddress)?;
		let to_account =
			T::AddressMapping::from_bech32(&to_address).ok_or(RootError::InvalidAddress)?;

		ctx.gas_meter()
			.consume_gas(T::DbWeight::get().reads(2).ref_time(), "")
			.map_err(|_| RootError::OutOfGas)?;

		for amt in amount.iter() {
			let transfer_amount =
				amt.amount.parse::<u128>().map_err(|_| RootError::InvalidCoins)?;

			if T::NativeDenom::get() == amt.denom {
				T::NativeAsset::transfer(
					&from_account,
					&to_account,
					transfer_amount.saturated_into(),
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| RootError::InsufficientFunds)?;

				ctx.gas_meter()
					.consume_gas(
						pallet_balances::weights::SubstrateWeight::<T>::transfer_keep_alive()
							.ref_time(),
						"",
					)
					.map_err(|_| RootError::OutOfGas)?;
			} else {
				let asset_id = T::AssetToDenom::convert(amt.denom.clone())
					.map_err(|_| RootError::InvalidCoins)?;
				T::Assets::transfer(
					asset_id,
					&from_account,
					&to_account,
					transfer_amount.saturated_into(),
					Preservation::Preserve,
				)
				.map_err(|_| RootError::InsufficientFunds)?;

				ctx.gas_meter()
					.consume_gas(
						pallet_assets::weights::SubstrateWeight::<T>::transfer_keep_alive()
							.ref_time(),
						"",
					)
					.map_err(|_| RootError::OutOfGas)?;
			}
		}

		let msg_event = pallet_cosmos_types::events::CosmosEvent {
			r#type: EVENT_TYPE_TRANSFER.into(),
			attributes: vec![
				EventAttribute { key: ATTRIBUTE_KEY_SENDER.into(), value: from_address.into() },
				EventAttribute { key: ATTRIBUTE_KEY_RECIPIENT.into(), value: to_address.into() },
				EventAttribute {
					key: ATTRIBUTE_KEY_AMOUNT.into(),
					value: amount_to_string(&amount).into(),
				},
			],
		};

		ctx.event_manager().emit_event(msg_event);

		Ok(())
	}
}
