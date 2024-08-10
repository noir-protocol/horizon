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

use core::str::FromStr;
use cosmos_sdk_proto::{
	cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin},
	prost::alloc::string::String,
	traits::Message,
	Any,
};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use pallet_balances::WeightInfo;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{
	address::address_from_bech32,
	events::{CosmosEvent, EventAttribute, ATTRIBUTE_KEY_AMOUNT, ATTRIBUTE_KEY_SENDER},
	msgservice::{MsgHandlerError, MsgHandlerErrorInfo},
};
use pallet_cosmos_x_bank_types::events::{ATTRIBUTE_KEY_RECIPIENT, EVENT_TYPE_TRANSFER};
use sp_runtime::{format_runtime_string, SaturatedConversion};
use sp_std::vec::Vec;

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
				error: MsgHandlerError::InvalidMsg,
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
			error: MsgHandlerError::InvalidMsg,
		})?;

		let to_addr = address_from_bech32(&to_address).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: MsgHandlerError::InvalidMsg,
		})?;

		let from_account = T::AddressMapping::into_account_id(from_addr);
		let to_account = T::AddressMapping::into_account_id(to_addr);
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

		for amt in amount.iter() {
			if T::NativeDenom::get() == amt.denom.as_bytes().to_vec() {
				T::Currency::transfer(
					&from_account,
					&to_account,
					amt.amount
						.parse::<u128>()
						.map_err(|_| MsgHandlerErrorInfo {
							weight: total_weight,
							error: MsgHandlerError::ParseAmountError,
						})?
						.saturated_into(),
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
					error: MsgHandlerError::InvalidMsg,
				});
			}
		}

		let msg_event = pallet_cosmos_types::events::CosmosEvent {
			r#type: EVENT_TYPE_TRANSFER.into(),
			attributes: sp_std::vec![
				EventAttribute { key: ATTRIBUTE_KEY_SENDER.into(), value: from_address.into() },
				EventAttribute { key: ATTRIBUTE_KEY_RECIPIENT.into(), value: to_address.into() },
				EventAttribute {
					key: ATTRIBUTE_KEY_AMOUNT.into(),
					value: amount_to_string(&amount).into()
				},
			],
		};

		Ok((total_weight, sp_std::vec![msg_event]))
	}
}

pub fn amount_to_string(amount: &[Coin]) -> String {
	let mut ret = String::from_str("").unwrap();
	for (i, coin) in amount.iter().enumerate() {
		ret.push_str(&coin.amount);
		ret.push_str(&coin.denom);
		if i < amount.len() - 1 {
			ret.push(',');
		}
	}
	ret
}

#[cfg(test)]
mod tests {
	use crate::msgs::amount_to_string;
	use core::str::FromStr;
	use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;

	#[test]
	fn amount_to_string_test() {
		let mut amounts = Vec::<Coin>::new();
		assert_eq!(amount_to_string(&amounts), "");

		amounts.push(Coin {
			denom: String::from_str("uatom").unwrap(),
			amount: String::from_str("1000").unwrap(),
		});
		assert_eq!(amount_to_string(&amounts), "1000uatom");

		amounts.push(Coin {
			denom: String::from_str("uatom").unwrap(),
			amount: String::from_str("2000").unwrap(),
		});

		assert_eq!(amount_to_string(&amounts), "1000uatom,2000uatom");
	}
}
