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

use frame_support::traits::Contains;
use pallet_cosmos_types::tx::Tx;
use pallet_cosmos_x::ante::AnteDecorator;
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};

pub struct KnownMsgDecorator<T>(sp_std::marker::PhantomData<T>);

impl<T> AnteDecorator for KnownMsgDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		for msg in &tx.body.messages {
			if !T::MsgFilter::contains(&msg.type_url) {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
			}
		}

		Ok(ValidTransaction::default())
	}
}
