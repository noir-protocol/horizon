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
use fp_self_contained::{CheckedExtrinsic, SelfContainedCall};
use frame_support::{assert_ok, dispatch::GetDispatchInfo, traits::fungible::Inspect};
use hp_account::CosmosSigner;
use pallet_cosmos_types::events::{CosmosEvent, EventAttribute};
use pallet_cosmos_x_bank_types::msgs::msg_send::MsgSend;
use sp_core::{ecdsa, Pair, H160};
use std::fs;

#[test]
fn pallet_cosmos_msg_send_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		System::reset_events();

		let alice = CosmosSigner(ecdsa::Pair::from_string("//Alice", None).unwrap().public());

		let tx_raw =  "CpoBCpcBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEncKLWNvc21vczFxZDY5bnV3ajk1Z3RhNGFramd5eHRqOXVqbXo0dzhlZG1xeXNxdxItY29zbW9zMW41amd4NjR6dzM4c3M3Nm16dXU0dWM3amV5cXcydmZqazYwZmR6GhcKBGFjZHQSDzEwMDAwMDAwMDAwMDAwMBJsCk4KRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiECChCRNB/lZkv6F4LV4Ed5aJBoyRawTLNl7DFTdVaE2aESBAoCCH8SGgoSCgRhY2R0EgoxMDQwMDAwMDAwEIDa8esEGkBgXIiPoBpecG7QpKDJPaztFogqvmxjDHF5ORfWBrOoSzf0+AAmch1CXrG4OmiKL0y8v9ITx0QzWYUc7ueXcdIm";
		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();
		let tx = Tx::decode(&mut &*tx_bytes).unwrap();

		let mut expected_balance = 1_000_000_000_000_000_000u128;
		assert_eq!(Balances::balance(&alice), expected_balance);

		let call = pallet_cosmos::Call::<Test>::transact { tx_bytes };
		let source = call.check_self_contained().unwrap().unwrap();
		let extrinsic = CheckedExtrinsic::<CosmosSigner, _, (), H160> {
			signed: fp_self_contained::CheckedSignature::SelfContained(source),
			function: RuntimeCall::Cosmos(call.clone()),
		};
		let dispatch_info = extrinsic.get_dispatch_info();

		assert_ok!(call.pre_dispatch_self_contained(&source, &dispatch_info, 0).unwrap());
		assert_ok!(extrinsic.function.apply_self_contained(alice).unwrap());

		let msg = tx.body.as_ref().unwrap().messages.first().unwrap();
		let msg = MsgSend::try_from(msg).unwrap();

		let amount = msg.amount.first().unwrap().amount.parse::<u128>().unwrap();
		expected_balance -= amount;

		let fee = tx.auth_info.as_ref().unwrap().fee.clone().unwrap();
		let fee_amount = fee.amount.first().unwrap().amount.parse::<u128>().unwrap();
		expected_balance -= fee_amount;

		assert_eq!(Balances::balance(&alice), expected_balance);
	});
}

#[test]
fn pallet_cosmos_msg_store_code_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		System::reset_events();

		let alice = CosmosSigner(ecdsa::Pair::from_string("//Alice", None).unwrap().public());

		let tx_raw = fs::read_to_string("./txs/msg_store_code").unwrap();
		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();

		let call = pallet_cosmos::Call::<Test>::transact { tx_bytes };
		let source = call.check_self_contained().unwrap().unwrap();
		let extrinsic = CheckedExtrinsic::<CosmosSigner, _, (), H160> {
			signed: fp_self_contained::CheckedSignature::SelfContained(source),
			function: RuntimeCall::Cosmos(call.clone()),
		};
		let dispatch_info = extrinsic.get_dispatch_info();

		assert_ok!(call.pre_dispatch_self_contained(&source, &dispatch_info, 0).unwrap());
		assert_ok!(extrinsic.function.apply_self_contained(alice).unwrap());

		let (_gas_wanted, _gas_used, events) = System::events()
			.into_iter()
			.find_map(|record| {
				if let RuntimeEvent::Cosmos(pallet_cosmos::Event::Executed {
					gas_wanted,
					gas_used,
					events,
				}) = record.event
				{
					Some((gas_wanted, gas_used, events))
				} else {
					None
				}
			})
			.unwrap();

		for CosmosEvent { r#type, attributes } in events.iter() {
			println!("type: {}", String::from_utf8_lossy(&r#type));

			for EventAttribute { key, value } in attributes.iter() {
				let key = String::from_utf8_lossy(key).to_string();
				let value = String::from_utf8_lossy(value).to_string();

				if key == "code_id" {
					assert_eq!(value, "1");
				}
				if key == "code_checksum" {
					assert_eq!(
						value,
						"db366741dcbad5f2e4933cda49133cd2a11fdb32b08c67cb1d22379bd392448e"
					);
				}
			}
		}
	});
}

#[test]
fn pallet_cosmos_msg_instantiate_contract2_test() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		System::reset_events();

		let alice = CosmosSigner(ecdsa::Pair::from_string("//Alice", None).unwrap().public());

		let tx_raw = fs::read_to_string("./txs/msg_store_code").unwrap();
		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();

		let call = pallet_cosmos::Call::<Test>::transact { tx_bytes };
		let source = call.check_self_contained().unwrap().unwrap();
		let extrinsic = CheckedExtrinsic::<CosmosSigner, _, (), H160> {
			signed: fp_self_contained::CheckedSignature::SelfContained(source),
			function: RuntimeCall::Cosmos(call.clone()),
		};
		let dispatch_info = extrinsic.get_dispatch_info();

		assert_ok!(call.pre_dispatch_self_contained(&source, &dispatch_info, 0).unwrap());
		assert_ok!(extrinsic.function.apply_self_contained(alice).unwrap());

		System::set_block_number(2);
		System::reset_events();

		let tx_raw = fs::read_to_string("./txs/msg_instantiate_contract2").unwrap();
		let tx_bytes = Base64::decode_vec(&tx_raw).unwrap();

		let call = pallet_cosmos::Call::<Test>::transact { tx_bytes };
		let source = call.check_self_contained().unwrap().unwrap();
		let extrinsic = CheckedExtrinsic::<CosmosSigner, _, (), H160> {
			signed: fp_self_contained::CheckedSignature::SelfContained(source),
			function: RuntimeCall::Cosmos(call.clone()),
		};
		let dispatch_info = extrinsic.get_dispatch_info();

		assert_ok!(call.pre_dispatch_self_contained(&source, &dispatch_info, 0).unwrap());
		assert_ok!(extrinsic.function.apply_self_contained(alice).unwrap());
	});
}
