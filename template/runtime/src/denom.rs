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

use crate::AssetId;
use cosmos_sdk_proto::prost::alloc::string::String;
use sp_runtime::traits::Convert;

pub struct DenomToAssetId;
impl Convert<String, Result<AssetId, ()>> for DenomToAssetId {
	fn convert(denom: String) -> Result<AssetId, ()> {
		let re = regex::Regex::new("^[a-zA-Z][a-zA-Z0-9/:._-]{2,15}$").unwrap();
		if !re.is_match(&denom) {
			return Err(());
		}

		let mut asset_id = [0u8; 16];
		asset_id[0..denom.len()].copy_from_slice(denom.as_bytes());

		Ok(u128::from_le_bytes(asset_id))
	}
}

#[cfg(test)]
mod tests {
	use super::DenomToAssetId;
	use cosmos_sdk_proto::prost::alloc::string::ToString;
	use frame_support::assert_err;
	use sp_runtime::traits::Convert;

	#[test]
	fn denom_to_asset_id_test() {
		let denom = "uatom".to_string();
		assert_eq!(DenomToAssetId::convert(denom).unwrap(), 470021333365);

		let denom = "deadbeafdeadbeaf".to_string();
		assert_eq!(
			DenomToAssetId::convert(denom).unwrap(),
			136086964684135438130477273377637754212
		);

		let denom = "".to_string();
		assert_err!(DenomToAssetId::convert(denom), ());

		let denom = "x".to_string();
		assert_err!(DenomToAssetId::convert(denom), ());

		let denom = "ab".to_string();
		assert_err!(DenomToAssetId::convert(denom), ());

		let denom = "deadbeafdeadbeafd".to_string();
		assert_err!(DenomToAssetId::convert(denom), ());
	}
}
