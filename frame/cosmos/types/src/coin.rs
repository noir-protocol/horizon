// This file is part of Horizon.

// Copyright (C) 2023 Haderech Pte. Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{error::DecodeError, traits::ToRuntimeString};
#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
use sp_runtime::format_runtime_string;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
pub struct Coin {
	pub denom: Vec<u8>,
	pub amount: u128,
}

#[cfg(feature = "std")]
impl From<&cosmrs::Coin> for Coin {
	fn from(coin: &cosmrs::Coin) -> Self {
		let denom = coin.denom.as_ref().as_bytes().to_vec();
		Self { denom, amount: coin.amount }
	}
}

impl ToRuntimeString for Coin {
	type Error = DecodeError;

	fn to_string(&self) -> Result<Vec<u8>, Self::Error> {
		let mut amount = format_runtime_string!("{}", self.amount).as_bytes().to_vec();
		amount.extend(self.denom.clone());
		Ok(amount)
	}
}

pub type Coins = Vec<Coin>;

impl ToRuntimeString for Coins {
	type Error = DecodeError;

	fn to_string(&self) -> Result<Vec<u8>, Self::Error> {
		if self.is_empty() {
			return Ok("".as_bytes().to_vec());
		}
		let mut coins = Vec::<Vec<u8>>::new();
		for (i, coin) in self.into_iter().enumerate() {
			coins.push(coin.to_string()?);
			if i < self.len() - 1 {
				coins.push(vec![b',']);
			}
		}
		Ok(coins.into_iter().flatten().collect())
	}
}

#[cfg(test)]
mod tests {
	use super::Coin;
	use crate::traits::ToRuntimeString;

	#[test]
	fn coin_to_string() {
		let coins = vec![];
		assert_eq!("".as_bytes().to_vec(), coins.to_string().unwrap());

		let coins = vec![Coin { denom: "uatom".as_bytes().to_vec(), amount: 1000000 }];
		assert_eq!("1000000uatom".as_bytes().to_vec(), coins.to_string().unwrap());

		let coins = vec![
			Coin { denom: "uatom".as_bytes().to_vec(), amount: 1000000 },
			Coin { denom: "acdt".as_bytes().to_vec(), amount: 1000000000000 },
		];
		assert_eq!(
			"1000000uatom,1000000000000acdt".as_bytes().to_vec(),
			coins.to_string().unwrap()
		);
	}
}
