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
use sp_runtime::transaction_validity::{TransactionValidity, ValidTransaction};

pub trait AnteDecorator {
	fn ante_handle(tx: &Tx, simulate: bool) -> TransactionValidity;
}

#[impl_trait_for_tuples::impl_for_tuples(1, 12)]
impl AnteDecorator for Tuple {
	fn ante_handle(tx: &Tx, simulate: bool) -> TransactionValidity {
		let valid = ValidTransaction::default();
		for_tuples!( #( let valid = valid.combine_with(Tuple::ante_handle(tx, simulate)?); )* );
		Ok(valid)
	}
}
