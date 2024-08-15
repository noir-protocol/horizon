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

use frame_support::{ensure, traits::fungibles::metadata::Inspect};
use sp_runtime::BoundedVec;

pub struct AssetsCallback<T>(sp_std::marker::PhantomData<T>);
impl<T> pallet_assets::AssetsCallback<T::AssetId, T::AccountId> for AssetsCallback<T>
where
	T: pallet_cosmos::Config,
{
	fn created(id: &T::AssetId, _owner: &T::AccountId) -> Result<(), ()> {
		let symbol = <T as pallet_cosmos::Config>::Assets::symbol(id.clone());
		let denom = BoundedVec::<u8, T::MaxDenomLimit>::try_from(symbol).map_err(|_| ())?;
		ensure!(pallet_cosmos::DenomAssetRouter::<T>::get(denom.clone()).is_none(), ());
		pallet_cosmos::DenomAssetRouter::<T>::insert(denom, id);

		Ok(())
	}

	fn destroyed(_id: &T::AssetId) -> Result<(), ()> {
		Ok(())
	}
}
