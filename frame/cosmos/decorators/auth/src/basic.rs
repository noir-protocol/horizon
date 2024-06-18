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

use hp_cosmos::Tx;
use pallet_cosmos_decorators::AnteDecorator;
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};
use sp_std::marker::PhantomData;

pub struct ValidateBasicDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator<T> for ValidateBasicDecorator<T>
where
	T: frame_system::Config,
{
	fn ante_handle(tx: &Tx) -> Result<(), TransactionValidityError> {
		if tx.signatures.is_empty() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
		}
		if tx.auth_info.signer_infos.len() != tx.signatures.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		Ok(())
	}
}
