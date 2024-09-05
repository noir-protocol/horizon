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

use crate::mock::*;
use base64ct::{Base64, Encoding};
use cosmos_sdk_proto::{cosmos::tx::v1beta1::Tx, prost::Message};
use frame_support::{assert_err, assert_ok, traits::fungible::Inspect};
use hp_account::CosmosSigner;
use hp_crypto::EcdsaExt;
use sp_core::{ecdsa, Pair};

#[test]
fn pallet_cosmos_msg_send_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		System::reset_events();

		let tx_raw =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();

		let alice = CosmosSigner(ecdsa::Pair::from_string("//Alice", None).unwrap().public());
		let alice_address = alice.to_cosmos_address().unwrap();

		assert_eq!(Balances::balance(&alice), 1_000_000_000_000_000_000);
		// assert_ok!(Cosmos::transact(pallet_cosmos::RawOrigin::CosmosTransaction(alice_address).into(), tx_bytes));
	});
}
