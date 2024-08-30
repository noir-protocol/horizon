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

use alloc::{string::String, vec::Vec};
use bech32::{Bech32, Hrp};
use core::str;
use hp_crypto::EcdsaExt;
use pallet_cosmos::AddressMapping;
use sp_core::Get;
use sp_runtime::traits::Convert;

pub struct AccountToAddr<T>(core::marker::PhantomData<T>);

impl<T> Convert<T::AccountId, String> for AccountToAddr<T>
where
	T: pallet_cosmos::Config,
	T::AccountId: EcdsaExt,
{
	fn convert(account: T::AccountId) -> String {
		// TODO: Handle errors
		let hrp = Hrp::parse(T::AddressPrefix::get()).unwrap();
		let address = account.to_cosmos_address().unwrap();

		bech32::encode::<Bech32>(hrp, address.as_bytes()).unwrap()
	}
}

impl<T> Convert<String, Result<T::AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(address: String) -> Result<T::AccountId, ()> {
		T::AddressMapping::from_bech32(&address).ok_or(())
	}
}

impl<T> Convert<Vec<u8>, Result<T::AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(address_raw: Vec<u8>) -> Result<T::AccountId, ()> {
		str::from_utf8(&address_raw)
			.map(T::AddressMapping::from_bech32)
			.map_err(|_| ())?
			.ok_or(())
	}
}
