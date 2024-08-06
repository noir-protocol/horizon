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

use cosmos_sdk_proto::cosmos::tx::v1beta1::Tx;
use pallet_cosmos_types::handler::AnteDecorator;
use sp_runtime::{
	traits::Get,
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
	SaturatedConversion,
};
use sp_std::marker::PhantomData;

pub struct ValidateBasicDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for ValidateBasicDecorator<T>
where
	T: frame_system::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		if tx.signatures.is_empty() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
		}
		let auth_info = tx
			.auth_info
			.as_ref()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

		if auth_info.signer_infos.len() == tx.signatures.len() {
			Ok(ValidTransaction::default())
		} else {
			Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))
		}
	}
}

pub struct TxTimeoutHeightDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for TxTimeoutHeightDecorator<T>
where
	T: frame_system::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let body = tx
			.body
			.as_ref()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?;

		if body.timeout_height > 0 &&
			frame_system::Pallet::<T>::block_number().saturated_into::<u64>() >
				body.timeout_height
		{
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale));
		}
		Ok(ValidTransaction::default())
	}
}

pub struct ValidateMemoDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for ValidateMemoDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let body = tx
			.body
			.as_ref()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?;

		if body.memo.len().saturated_into::<u64>() > T::MaxMemoCharacters::get() {
			Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
		} else {
			Ok(ValidTransaction::default())
		}
	}
}
