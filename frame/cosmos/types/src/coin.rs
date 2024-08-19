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

use core::str::FromStr;
use cosmos_sdk_proto::{cosmos::base::v1beta1::Coin, prost::alloc::string::String};

pub fn amount_to_string(amount: &[Coin]) -> String {
	let mut ret = String::from_str("").unwrap();
	for (i, coin) in amount.iter().enumerate() {
		ret.push_str(&coin.amount);
		ret.push_str(&coin.denom);
		if i < amount.len() - 1 {
			ret.push(',');
		}
	}
	ret
}

#[cfg(test)]
mod tests {
	use crate::coin::amount_to_string;
	use core::str::FromStr;
	use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;

	#[test]
	fn amount_to_string_test() {
		let mut amounts = Vec::<Coin>::new();
		assert_eq!(amount_to_string(&amounts), "");

		amounts.push(Coin {
			denom: String::from_str("uatom").unwrap(),
			amount: String::from_str("1000").unwrap(),
		});
		assert_eq!(amount_to_string(&amounts), "1000uatom");

		amounts.push(Coin {
			denom: String::from_str("uatom").unwrap(),
			amount: String::from_str("2000").unwrap(),
		});

		assert_eq!(amount_to_string(&amounts), "1000uatom,2000uatom");
	}
}