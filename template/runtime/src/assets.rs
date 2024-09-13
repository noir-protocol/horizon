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

use alloc::string::{String, ToString};
use frame_support::{ensure, traits::fungibles::metadata::Inspect};
use sp_core::Get;
use sp_runtime::{traits::Convert, BoundedVec};

pub struct AssetsCallback<T>(core::marker::PhantomData<T>);
impl<T> pallet_assets::AssetsCallback<T::AssetId, T::AccountId> for AssetsCallback<T>
where
	T: pallet_cosmos::Config,
{
	fn created(id: &T::AssetId, _owner: &T::AccountId) -> Result<(), ()> {
		let symbol = T::Assets::symbol(id.clone());
		ensure!(!symbol.is_empty(), ());

		let denom = BoundedVec::<u8, T::MaxDenomLimit>::try_from(symbol).map_err(|_| ())?;

		ensure!(pallet_cosmos::DenomAssetRouter::<T>::get(denom.clone()).is_none(), ());
		ensure!(pallet_cosmos::AssetDenomRouter::<T>::get(id.clone()).is_none(), ());

		pallet_cosmos::DenomAssetRouter::<T>::insert(denom.clone(), id);
		pallet_cosmos::AssetDenomRouter::<T>::insert(id, denom);

		Ok(())
	}

	fn destroyed(id: &T::AssetId) -> Result<(), ()> {
		if let Some(denom) = pallet_cosmos::AssetDenomRouter::<T>::take(id) {
			pallet_cosmos::DenomAssetRouter::<T>::remove(denom);
		}

		Ok(())
	}
}

pub struct AssetToDenom<T>(core::marker::PhantomData<T>);
impl<T> Convert<String, Result<T::AssetId, ()>> for AssetToDenom<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(denom: String) -> Result<T::AssetId, ()> {
		if denom == T::NativeDenom::get() {
			Ok(T::NativeAssetId::get())
		} else {
			let denom: BoundedVec<u8, T::MaxDenomLimit> =
				denom.as_bytes().to_vec().try_into().map_err(|_| ())?;
			pallet_cosmos::DenomAssetRouter::<T>::get(denom).ok_or(())
		}
	}
}

impl<T> Convert<T::AssetId, String> for AssetToDenom<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(asset_id: T::AssetId) -> String {
		if asset_id == T::NativeAssetId::get() {
			T::NativeDenom::get().to_string()
		} else {
			// TODO: Handle option
			let denom = pallet_cosmos::AssetDenomRouter::<T>::get(asset_id).unwrap().to_vec();
			String::from_utf8(denom).unwrap()
		}
	}
}
