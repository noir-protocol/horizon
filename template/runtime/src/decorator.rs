// This file is part of Hrozion.

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

use frame_support::pallet_prelude::*;
use pallet_cosmos_auth::{
	SigVerificationDecorator, TxTimeoutHeightDecorator, ValidateBasicDecorator,
};
use pallet_cosmos_decorators::AnteDecorator;

pub struct AnteDecorators;
impl<T> pallet_cosmos_decorators::AnteDecorators<T> for AnteDecorators
where
	T: frame_system::Config,
{
	fn ante_handle(tx: &hp_cosmos::Tx) -> Result<(), TransactionValidityError> {
		ValidateBasicDecorator::<T>::ante_handle(tx)?;
		TxTimeoutHeightDecorator::<T>::ante_handle(tx)?;
		SigVerificationDecorator::<T>::ante_handle(tx)?;

		Ok(())
	}
}
