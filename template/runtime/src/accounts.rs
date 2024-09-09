// This file is part of Horizion.

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

use crate::AccountId;
use alloc::{string::String, vec::Vec};
use bech32::{Bech32, Hrp};
use core::marker::PhantomData;
use hp_account::CosmosSigner;
use hp_crypto::EcdsaExt;
use pallet_cosmos::AddressMapping;
use sp_core::{crypto::UncheckedFrom, Get, H160, H256};
use sp_runtime::traits::Convert;

pub struct AccountToAddr<T>(PhantomData<T>);

impl<T> Convert<AccountId, String> for AccountToAddr<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(account: AccountId) -> String {
		// TODO: Handle error
		let data = if *account.0 .0.first().unwrap() == 0 {
			&account.0 .0[1..]
		} else {
			&account.to_cosmos_address().unwrap().0[..]
		};

		let hrp = Hrp::parse(T::AddressPrefix::get()).unwrap();
		bech32::encode::<Bech32>(hrp, data).unwrap()
	}
}

impl<T> Convert<String, Result<AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config<AccountId = CosmosSigner>,
{
	fn convert(address: String) -> Result<AccountId, ()> {
		bech32::decode(&address)
			.map(|(_hrp, data)| Self::convert(data))
			.map_err(|_| ())?
	}
}

impl<T> Convert<Vec<u8>, Result<AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config<AccountId = CosmosSigner>,
{
	fn convert(address_raw: Vec<u8>) -> Result<AccountId, ()> {
		// Cosmos address length is 20, contract address is 32.
		let account = match address_raw.len() {
			20 => T::AddressMapping::from_address_raw(H160::from_slice(&address_raw)),
			32 => AccountId::unchecked_from(H256::from_slice(&address_raw)),
			_ => return Err(()),
		};

		Ok(account)
	}
}
