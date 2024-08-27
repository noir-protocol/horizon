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

use cosmos_sdk_proto::prost::alloc::string::String;
use sp_runtime::{traits::Convert, BoundedVec};

pub struct DenomToAsset<T>(sp_std::marker::PhantomData<T>);
impl<T> Convert<String, Result<T::AssetId, ()>> for DenomToAsset<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(denom: String) -> Result<T::AssetId, ()> {
		let denom = BoundedVec::<u8, T::MaxDenomLimit>::try_from(denom.as_bytes().to_vec())
			.map_err(|_| ())?;
		pallet_cosmos::DenomAssetRouter::<T>::get(denom).ok_or(())
	}
}
