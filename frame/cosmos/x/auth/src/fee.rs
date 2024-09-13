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
use core::marker::PhantomData;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{Fee, Tx};
use frame_support::{
	pallet_prelude::InvalidTransaction,
	traits::{
		fungibles::Balanced,
		tokens::{Fortitude, Precision, Preservation},
		Currency, ExistenceRequirement, WithdrawReasons,
	},
};
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{
	address::acc_address_from_bech32,
	coin::amount_to_string,
	events::{
		CosmosEvent, EventAttribute, ATTRIBUTE_KEY_FEE, ATTRIBUTE_KEY_FEE_PAYER, EVENT_TYPE_TX,
	},
	handler::AnteDecorator,
};
use pallet_cosmos_x_auth_signing::sign_verifiable_tx::traits::SigVerifiableTx;
use sp_core::{Get, H160};
use sp_runtime::{
	traits::{Convert, Zero},
	transaction_validity::{TransactionValidity, ValidTransaction},
	SaturatedConversion,
};

pub struct DeductFeeDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for DeductFeeDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, simulate: bool) -> TransactionValidity {
		let fee = tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(InvalidTransaction::Call)?;

		if !simulate &&
			!frame_system::Pallet::<T>::block_number().is_zero() &&
			fee.gas_limit.is_zero()
		{
			return Err(InvalidTransaction::Call.into());
		}

		// TODO: Implements txFeeChecker

		Self::check_deduct_fee(tx)?;

		Ok(ValidTransaction::default())
	}
}

impl<T> DeductFeeDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn check_deduct_fee(tx: &Tx) -> TransactionValidity {
		let fee_payer = T::SigVerifiableTx::fee_payer(tx).map_err(|_| InvalidTransaction::Call)?;

		let fee = tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(InvalidTransaction::Call)?;

		// TODO: Fee granter not supported
		if !fee.granter.is_empty() {
			return Err(InvalidTransaction::Call.into());
		}

		let (_hrp, address_raw) =
			acc_address_from_bech32(&fee_payer).map_err(|_| InvalidTransaction::BadSigner)?;
		if address_raw.len() != 20 {
			return Err(InvalidTransaction::BadSigner.into());
		}
		let deduct_fees_from = T::AddressMapping::into_account_id(H160::from_slice(&address_raw));

		// TODO: Check fee is zero
		if !fee.amount.is_empty() {
			Self::deduct_fees(&deduct_fees_from, fee)?;
		}

		pallet_cosmos::Pallet::<T>::deposit_event(pallet_cosmos::Event::AnteHandled(vec![
			CosmosEvent {
				r#type: EVENT_TYPE_TX.into(),
				attributes: vec![
					EventAttribute {
						key: ATTRIBUTE_KEY_FEE.into(),
						value: amount_to_string(&fee.amount).into(),
					},
					EventAttribute { key: ATTRIBUTE_KEY_FEE_PAYER.into(), value: fee_payer.into() },
				],
			},
		]));

		Ok(ValidTransaction::default())
	}

	fn deduct_fees(acc: &T::AccountId, fee: &Fee) -> TransactionValidity {
		for amt in fee.amount.iter() {
			let amount = amt.amount.parse::<u128>().map_err(|_| InvalidTransaction::Call)?;

			if amt.denom == T::NativeDenom::get() {
				let _imbalance = T::NativeAsset::withdraw(
					acc,
					amount.saturated_into(),
					WithdrawReasons::TRANSACTION_PAYMENT,
					ExistenceRequirement::KeepAlive,
				)
				.map_err(|_| InvalidTransaction::Payment)?;

			// TODO: Resolve imbalance
			} else {
				let asset_id = T::AssetToDenom::convert(amt.denom.clone())
					.map_err(|_| InvalidTransaction::Call)?;

				let _imbalance = T::Assets::withdraw(
					asset_id,
					acc,
					amount.saturated_into(),
					Precision::Exact,
					Preservation::Preserve,
					Fortitude::Polite,
				)
				.map_err(|_| InvalidTransaction::Payment)?;

				// TODO: Resolve imbalance
			}
		}

		Ok(ValidTransaction::default())
	}
}
