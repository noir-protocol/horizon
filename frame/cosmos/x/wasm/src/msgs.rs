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

use alloc::{string::ToString, vec, vec::Vec};
use bech32::{Bech32, Hrp};
use core::{marker::PhantomData, str::FromStr};
use core2::io::Read;
use cosmos_sdk_proto::{
	cosmwasm::wasm::v1::{
		MsgExecuteContract, MsgInstantiateContract2, MsgMigrateContract, MsgStoreCode,
		MsgUpdateAdmin,
	},
	prost::Message,
	Any,
};
use frame_support::{traits::Get, weights::Weight};
use hp_crypto::EcdsaExt;
use libflate::gzip::Decoder;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{
	address::address_from_bech32,
	errors::RootError,
	events::{CosmosEvent, EventAttribute},
	msgservice::MsgHandlerErrorInfo,
};
use pallet_cosmos_x_wasm_types::{
	errors::WasmError,
	events::{
		ATTRIBUTE_KEY_CHECKSUM, ATTRIBUTE_KEY_CODE_ID, ATTRIBUTE_KEY_CONTRACT_ADDR,
		EVENT_TYPE_INSTANTIATE, EVENT_TYPE_STORE_CODE,
	},
};
use pallet_cosmwasm::{
	runtimes::vm::InitialStorageMutability,
	types::{
		CodeIdentifier, ContractCodeOf, ContractLabelOf, ContractMessageOf, ContractSaltOf, FundsOf,
	},
};
use sp_runtime::{traits::Convert, SaturatedConversion};

pub struct MsgStoreCodeHandler<T>(PhantomData<T>);

impl<T> Default for MsgStoreCodeHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgStoreCodeHandler<T>
where
	T: pallet_cosmos::Config + pallet_cosmwasm::Config,
{
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		// TODO: Apply actual weights
		let total_weight = Weight::zero();

		let MsgStoreCode { sender, wasm_byte_code, instantiate_permission: _ } =
			MsgStoreCode::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

		let who =
			address_from_bech32(&sender)
				.map(T::AddressMapping::into_account_id)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: RootError::TxDecodeError.into(),
				})?;

		let mut decoder = Decoder::new(&wasm_byte_code[..]).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: WasmError::CreateFailed.into(),
		})?;
		let mut decoded_code = Vec::new();
		decoder.read_to_end(&mut decoded_code).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: WasmError::CreateFailed.into(),
		})?;

		let code: ContractCodeOf<T> = decoded_code.try_into().map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: WasmError::CreateFailed.into(),
		})?;

		let (code_hash, code_id) =
			pallet_cosmwasm::Pallet::<T>::do_upload(&who, code).map_err(|_| {
				MsgHandlerErrorInfo { weight: total_weight, error: WasmError::CreateFailed.into() }
			})?;

		// TODO: Duplicated events emitted
		let msg_event = CosmosEvent {
			r#type: EVENT_TYPE_STORE_CODE.into(),
			attributes: vec![
				EventAttribute {
					key: ATTRIBUTE_KEY_CODE_ID.into(),
					value: code_id.to_string().into(),
				},
				EventAttribute {
					key: ATTRIBUTE_KEY_CHECKSUM.into(),
					value: hex::encode(code_hash.0).into(),
				},
			],
		};

		Ok((total_weight, vec![msg_event]))
	}
}

pub struct MsgInstantiateContract2Handler<T>(PhantomData<T>);

impl<T> Default for MsgInstantiateContract2Handler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgInstantiateContract2Handler<T>
where
	T: pallet_cosmos::Config + pallet_cosmwasm::Config,
	T::AccountId: EcdsaExt,
{
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let total_weight = Weight::zero();

		let MsgInstantiateContract2 {
			sender,
			admin,
			code_id,
			label,
			msg,
			funds: coins,
			salt,
			fix_msg: _,
		} = MsgInstantiateContract2::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: RootError::TxDecodeError.into(),
		})?;

		if sender.is_empty() {
			return Err(MsgHandlerErrorInfo {
				weight: total_weight,
				error: WasmError::Empty.into(),
			});
		}

		let who =
			address_from_bech32(&sender)
				.map(T::AddressMapping::into_account_id)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: RootError::TxDecodeError.into(),
				})?;

		// TODO: Get gas limit from tx
		let mut shared = pallet_cosmwasm::Pallet::<T>::do_create_vm_shared(
			0,
			InitialStorageMutability::ReadWrite,
		);
		let code_identifier = CodeIdentifier::CodeId(code_id);

		let salt: ContractSaltOf<T> = salt.try_into().map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: RootError::TxDecodeError.into(),
		})?;

		let admin = if !admin.is_empty() {
			let admin = address_from_bech32(&admin)
				.map(T::AddressMapping::into_account_id)
				.map_err(|_| MsgHandlerErrorInfo {
					weight: total_weight,
					error: RootError::TxDecodeError.into(),
				})?;
			Some(admin)
		} else {
			None
		};

		let label: ContractLabelOf<T> = label.as_bytes().to_vec().try_into().map_err(|_| {
			MsgHandlerErrorInfo { weight: total_weight, error: RootError::TxDecodeError.into() }
		})?;

		let mut funds = FundsOf::<T>::default();
		for coin in coins.iter() {
			let asset_id = <T as pallet_cosmwasm::Config>::AssetToDenom::convert(
				coin.denom.clone(),
			)
			.map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;
			let amount = u128::from_str(&coin.amount).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

			funds.try_insert(asset_id, (amount.saturated_into(), true)).map_err(|_| {
				MsgHandlerErrorInfo { weight: total_weight, error: RootError::TxDecodeError.into() }
			})?;
		}

		let message: ContractMessageOf<T> = msg.try_into().map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: RootError::TxDecodeError.into(),
		})?;

		let contract = pallet_cosmwasm::Pallet::<T>::do_instantiate(
			&mut shared,
			who,
			code_identifier,
			salt,
			admin,
			label,
			funds,
			message,
		)
		.map_err(|_| MsgHandlerErrorInfo {
			weight: total_weight,
			error: WasmError::InstantiateFailed.into(),
		})?;
		let contract_address = contract.to_cosm_address().ok_or(MsgHandlerErrorInfo {
			weight: total_weight,
			error: WasmError::InstantiateFailed.into(),
		})?;

		let hrp = Hrp::parse(T::AddressPrefix::get()).unwrap();
		let contract_address = bech32::encode::<Bech32>(hrp, contract_address.as_bytes()).unwrap();

		// TODO: Duplicated events emitted
		let msg_event = CosmosEvent {
			r#type: EVENT_TYPE_INSTANTIATE.into(),
			attributes: vec![
				EventAttribute {
					key: ATTRIBUTE_KEY_CONTRACT_ADDR.into(),
					value: contract_address.into(),
				},
				EventAttribute {
					key: ATTRIBUTE_KEY_CODE_ID.into(),
					value: code_id.to_string().into(),
				},
			],
		};

		Ok((total_weight, vec![msg_event]))
	}
}

pub struct MsgExecuteContractHandler<T>(PhantomData<T>);

impl<T> Default for MsgExecuteContractHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgExecuteContractHandler<T> {
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let total_weight = Weight::zero();

		let MsgExecuteContract { sender: _, contract: _, msg: _, funds: _ } =
			MsgExecuteContract::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

		// TODO: Implements execute contract with pallet_cosmwasm
		Err(MsgHandlerErrorInfo { weight: total_weight, error: RootError::UnknownRequest.into() })
	}
}

pub struct MsgMigrateContractHandler<T>(PhantomData<T>);

impl<T> Default for MsgMigrateContractHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgMigrateContractHandler<T> {
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let total_weight = Weight::zero();

		let MsgMigrateContract { sender: _, contract: _, code_id: _, msg: _ } =
			MsgMigrateContract::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

		// TODO: Implements migrate contract with pallet_cosmwasm
		Err(MsgHandlerErrorInfo { weight: total_weight, error: RootError::UnknownRequest.into() })
	}
}

pub struct MsgUpdateAdminHandler<T>(PhantomData<T>);

impl<T> Default for MsgUpdateAdminHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgUpdateAdminHandler<T> {
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let total_weight = Weight::zero();

		let MsgUpdateAdmin { sender: _, new_admin: _, contract: _ } =
			MsgUpdateAdmin::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

		// TODO: Implements update admin with pallet_cosmwasm
		Err(MsgHandlerErrorInfo { weight: total_weight, error: RootError::UnknownRequest.into() })
	}
}
