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

use super::*;
use alloc::{boxed::Box, string::String, vec::Vec};
use bech32::{Bech32, Hrp};
use core::marker::PhantomData;
use cosmos_sdk_proto::{
	cosmos::{
		bank::v1beta1::MsgSend,
		tx::v1beta1::{
			mode_info::{Single, Sum},
			ModeInfo, SignDoc, TxRaw,
		},
	},
	cosmwasm::wasm::v1::{
		MsgExecuteContract, MsgInstantiateContract2, MsgMigrateContract, MsgStoreCode,
		MsgUpdateAdmin,
	},
	Any,
};
use frame_support::{
	derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, Contains},
	PalletId,
};
use hp_account::CosmosSigner;
use hp_crypto::EcdsaExt;
use pallet_cosmos::{
	config_preludes::{
		AddressPrefix, ChainId, Context, MaxDenomLimit, MaxMemoCharacters, MsgFilter, NativeDenom,
		TxSigLimit, WeightToGas,
	},
	AddressMapping,
};
use pallet_cosmos_types::msgservice::MsgHandler;
use pallet_cosmos_x_auth_migrations::legacytx::stdsign::StdSignDoc;
use pallet_cosmos_x_auth_signing::{
	any_match,
	sign_mode_handler::{SignModeHandlerError, SignerData},
	sign_verifiable_tx::SigVerifiableTxError,
};
use pallet_cosmos_x_bank::msgs::MsgSendHandler;
use pallet_cosmos_x_bank_types::msgs::msg_send;
use pallet_cosmos_x_wasm::msgs::{
	MsgExecuteContractHandler, MsgInstantiateContract2Handler, MsgMigrateContractHandler,
	MsgStoreCodeHandler, MsgUpdateAdminHandler,
};
use pallet_cosmos_x_wasm_types::tx::{
	msg_execute_contract, msg_instantiate_contract2, msg_migrate_contract, msg_store_code,
	msg_update_admin,
};
use pallet_cosmwasm::instrument::CostRules;
use serde_json::{Map, Value};
use sp_core::{crypto::UncheckedFrom, ecdsa, ConstU128, ConstU32, ConstU64, Hasher, H160, H256};
use sp_runtime::traits::{BlakeTwo256, Convert, IdentityLookup};

type Block = frame_system::mocking::MockBlock<Test>;
type AssetId = u64;
type AccountId = CosmosSigner;
type Balance = u128;
type Hash = sp_core::H256;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		Cosmos: pallet_cosmos,
		Cosmwasm: pallet_cosmwasm,
		Timestamp: pallet_timestamp,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = CosmosSigner;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type AssetId = AssetId;
	type AssetIdParameter = AssetId;
	type Balance = Balance;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Freezer = ();
	type AssetDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<5>;
	type WeightInfo = ();
}

pub struct MsgServiceRouter<T>(PhantomData<T>);
impl<T, Context> pallet_cosmos_types::msgservice::MsgServiceRouter<Context> for MsgServiceRouter<T>
where
	T: frame_system::Config + pallet_cosmos::Config + pallet_cosmwasm::Config,
	T::AccountId: EcdsaExt,
	Context: store::Context,
{
	fn route(msg: &Any) -> Option<Box<dyn MsgHandler<Context>>> {
		any_match!(
			msg, {
				MsgSend => Some(Box::<MsgSendHandler<T>>::default()),
				MsgStoreCode => Some(Box::<MsgStoreCodeHandler<T>>::default()),
				MsgInstantiateContract2 => Some(Box::<MsgInstantiateContract2Handler<T>>::default()),
				MsgExecuteContract => Some(Box::<MsgExecuteContractHandler<T>>::default()),
				MsgMigrateContract => Some(Box::<MsgMigrateContractHandler<T>>::default()),
				MsgUpdateAdmin => Some(Box::<MsgUpdateAdminHandler<T>>::default()),
			},
			None
		)
	}
}

pub struct SigVerifiableTx;
impl pallet_cosmos_x_auth_signing::sign_verifiable_tx::SigVerifiableTx for SigVerifiableTx {
	fn get_signers(tx: &Tx) -> Result<Vec<String>, SigVerifiableTxError> {
		let mut signers = Vec::<String>::new();

		let body = tx.body.as_ref().ok_or(SigVerifiableTxError::EmptyTxBody)?;
		for msg in body.messages.iter() {
			let msg_signers = any_match!(
				msg, {
					MsgSend => MsgSend::decode(&mut &*msg.value).as_ref().map(msg_send::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgStoreCode => MsgStoreCode::decode(&mut &*msg.value).as_ref().map(msg_store_code::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgInstantiateContract2 => MsgInstantiateContract2::decode(&mut &*msg.value).as_ref().map(msg_instantiate_contract2::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgExecuteContract => MsgExecuteContract::decode(&mut &*msg.value).as_ref().map(msg_execute_contract::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgMigrateContract => MsgMigrateContract::decode(&mut &*msg.value).as_ref().map(msg_migrate_contract::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
					MsgUpdateAdmin => MsgUpdateAdmin::decode(&mut &*msg.value).as_ref().map(msg_update_admin::get_signers).map_err(|_| SigVerifiableTxError::InvalidMsg),
				},
				Err(SigVerifiableTxError::InvalidMsg)
			)?;

			for msg_signer in msg_signers.iter() {
				if !signers.contains(msg_signer) {
					signers.push(msg_signer.clone());
				}
			}
		}

		let fee_payer = &tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(SigVerifiableTxError::EmptyFee)?
			.payer;

		if !fee_payer.is_empty() && !signers.contains(fee_payer) {
			signers.push(fee_payer.clone());
		}

		Ok(signers)
	}

	fn fee_payer(tx: &Tx) -> Result<String, SigVerifiableTxError> {
		let fee = tx
			.auth_info
			.as_ref()
			.and_then(|auth_info| auth_info.fee.as_ref())
			.ok_or(SigVerifiableTxError::EmptyFee)?;

		let fee_payer = if fee.payer.is_empty() {
			Self::get_signers(tx)?
				.first()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.clone()
		} else {
			fee.payer.clone()
		};

		Ok(fee_payer)
	}

	fn sequence(tx: &Tx) -> Result<u64, SigVerifiableTxError> {
		let auth_info = tx.auth_info.as_ref().ok_or(SigVerifiableTxError::EmptyAuthInfo)?;
		let fee = auth_info.fee.as_ref().ok_or(SigVerifiableTxError::EmptyFee)?;

		let sequence = if !fee.payer.is_empty() {
			auth_info
				.signer_infos
				.first()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.sequence
		} else {
			// TODO: Verify that the last signer is the fee payer.
			auth_info
				.signer_infos
				.last()
				.ok_or(SigVerifiableTxError::EmptySigners)?
				.sequence
		};

		Ok(sequence)
	}
}

pub struct HashedAddressMapping<T, H>(PhantomData<(T, H)>);

impl<T, H> AddressMapping<T::AccountId> for HashedAddressMapping<T, H>
where
	T: pallet_cosmos_accounts::Config,
	T::AccountId: From<CosmosSigner> + EcdsaExt,
	H: Hasher<Out = H256>,
{
	fn from_address_raw(address: H160) -> T::AccountId {
		if let Some(x) = pallet_cosmos_accounts::Connections::<T>::get(address) {
			return x;
		}
		let mut data = [0u8; 25];
		data[0..5].copy_from_slice(b"cosm:");
		data[5..25].copy_from_slice(&address[..]);
		let hash = H::hash(&data);

		let mut interim = [0u8; 33];
		interim[1..33].copy_from_slice(&hash.0[..]);

		CosmosSigner(ecdsa::Public(interim)).into()
	}

	fn from_bech32(address: &str) -> Option<T::AccountId> {
		let (_hrp, address_raw) = acc_address_from_bech32(address).ok()?;
		if address_raw.len() != 20 {
			return None;
		}

		Some(Self::from_address_raw(H160::from_slice(&address_raw)))
	}
}

pub struct SignModeHandler;

impl pallet_cosmos_x_auth_signing::sign_mode_handler::SignModeHandler for SignModeHandler {
	fn get_sign_bytes(
		mode: &ModeInfo,
		data: &SignerData,
		tx: &Tx,
	) -> Result<Vec<u8>, SignModeHandlerError> {
		let sum = mode.sum.as_ref().ok_or(SignModeHandlerError::EmptyModeInfo)?;
		let sign_bytes = match sum {
			Sum::Single(Single { mode }) => match mode {
				1 /* SIGN_MODE_DIRECT */ => {
					let tx_raw = TxRaw::decode(&mut &*tx.encode_to_vec()).map_err(|_| SignModeHandlerError::DecodeTxError)?;
					SignDoc {
						body_bytes: tx_raw.body_bytes,
						auth_info_bytes: tx_raw.auth_info_bytes,
						chain_id: data.chain_id.clone(),
						account_number: data.account_number,
					}.encode_to_vec()
				},
				127 /* SIGN_MODE_LEGACY_AMINO_JSON */ => {
					let fee = tx.auth_info.as_ref().and_then(|auth_info| auth_info.fee.as_ref()).ok_or(SignModeHandlerError::EmptyFee)?;
					let body = tx.body.as_ref().ok_or(SignModeHandlerError::EmptyTxBody)?;

					let mut coins = Vec::<Value>::new();
					for amt in fee.amount.iter() {
						let mut coin = Map::new();
						coin.insert("amount".to_string(), Value::String(amt.amount.clone()));
						coin.insert("denom".to_string(), Value::String(amt.denom.clone()));

						coins.push(Value::Object(coin));
					}

					let mut std_fee = Map::new();
					std_fee.insert("gas".to_string(), Value::String(fee.gas_limit.to_string()));
					std_fee.insert("amount".to_string(), Value::Array(coins));

					let mut msgs = Vec::<Value>::new();
					for msg in body.messages.iter() {
						let sign_msg = any_match!(
							msg, {
								MsgSend => MsgSend::decode(&mut &*msg.value).as_ref().map(msg_send::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgStoreCode => MsgStoreCode::decode(&mut &*msg.value).as_ref().map(msg_store_code::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgInstantiateContract2 => MsgInstantiateContract2::decode(&mut &*msg.value).as_ref().map(msg_instantiate_contract2::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgExecuteContract => MsgExecuteContract::decode(&mut &*msg.value).as_ref().map(msg_execute_contract::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgMigrateContract => MsgMigrateContract::decode(&mut &*msg.value).as_ref().map(msg_migrate_contract::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
								MsgUpdateAdmin => MsgUpdateAdmin::decode(&mut &*msg.value).as_ref().map(msg_update_admin::get_sign_bytes).map_err(|_| SignModeHandlerError::InvalidMsg),
							},
							Err(SignModeHandlerError::InvalidMsg))?;

						msgs.push(sign_msg);
					}

					let sign_doc = StdSignDoc {
						account_number: data.account_number.to_string(),
						chain_id: data.chain_id.clone(),
						fee: Value::Object(std_fee),
						memo: body.memo.clone(),
						msgs,
						sequence: data.sequence.to_string(),
					};
					serde_json::to_value(sign_doc).map_err(|_| SignModeHandlerError::SerializeError)?.to_string().as_bytes().to_vec()
				},
				_ => return Err(SignModeHandlerError::InvalidMode),
			},
			_ => return Err(SignModeHandlerError::InvalidMode),
		};

		Ok(sign_bytes)
	}
}

#[derive_impl(pallet_cosmos::config_preludes::TestDefaultConfig)]
impl pallet_cosmos::Config for Test {
	type AddressMapping = HashedAddressMapping<Self::AccountId, BlakeTwo256>;
	type NativeAsset = Balances;
	type Assets = Assets;
	type RuntimeEvent = RuntimeEvent;
	type AnteHandler = ();
	type Balance = Balance;
	type AssetId = AssetId;
	type MaxMemoCharacters = MaxMemoCharacters;
	type NativeDenom = NativeDenom;
	type ChainId = ChainId;
	type MsgFilter = MsgFilter;
	type WeightToGas = WeightToGas;
	type TxSigLimit = TxSigLimit;
	type MaxDenomLimit = MaxDenomLimit;
	type AddressPrefix = AddressPrefix;
	type Context = Context;
	type MsgServiceRouter = MsgServiceRouter<Test>;
	type SigVerifiableTx = SigVerifiableTx;
	type WeightInfo = pallet_cosmos::weights::CosmosWeight<Test>;
	type AssetToDenom = AssetToDenom;
	type SignModeHandler = SignModeHandler;
}

parameter_types! {
	pub const CosmwasmPalletId: PalletId = PalletId(*b"cosmwasm");
	pub const MaxContractLabelSize: u32 = 64;
	pub const MaxContractTrieIdSize: u32 = Hash::len_bytes() as u32;
	pub const MaxInstantiateSaltSize: u32 = 128;
	pub const MaxFundsAssets: u32 = 32;
	pub const CodeTableSizeLimit: u32 = 4096;
	pub const CodeGlobalVariableLimit: u32 = 256;
	pub const CodeParameterLimit: u32 = 128;
	pub const CodeBranchTableSizeLimit: u32 = 256;
	pub const CodeStorageByteDeposit: u32 = 1_000_000;
	pub const ContractStorageByteReadPrice: u32 = 1;
	pub const ContractStorageByteWritePrice: u32 = 1;
	pub WasmCostRules: CostRules<Test> = Default::default();
}

pub struct AssetToDenom;
impl Convert<String, Result<AssetId, ()>> for AssetToDenom {
	fn convert(denom: String) -> Result<AssetId, ()> {
		denom.parse::<AssetId>().map_err(|_| ())
	}
}
impl Convert<AssetId, String> for AssetToDenom {
	fn convert(asset_id: AssetId) -> String {
		asset_id.to_string()
	}
}

struct AccountToAddr<T>(PhantomData<T>);
impl<T> Convert<AccountId, String> for AccountToAddr<T>
where
	T: pallet_cosmos::Config,
{
	fn convert(account: AccountId) -> String {
		let data = if *account.0 .0.first().unwrap() == 0 {
			&account.0 .0[1..]
		} else {
			&account.to_cosmos_address().unwrap().0[..]
		};

		let hrp = Hrp::parse(T::AddressPrefix::get()).unwrap();
		bech32::encode::<Bech32>(hrp, data).unwrap()
	}
}
impl<T> Convert<String, Result<AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config<AccountId = CosmosSigner>,
{
	fn convert(address: String) -> Result<AccountId, ()> {
		bech32::decode(&address)
			.map(|(_hrp, data)| Self::convert(data))
			.map_err(|_| ())?
	}
}
impl<T> Convert<Vec<u8>, Result<AccountId, ()>> for AccountToAddr<T>
where
	T: pallet_cosmos::Config<AccountId = CosmosSigner>,
{
	fn convert(address_raw: Vec<u8>) -> Result<AccountId, ()> {
		// Cosmos address length is 20, contract address is 32.
		let account = match address_raw.len() {
			20 => T::AddressMapping::from_address_raw(H160::from_slice(&address_raw)),
			32 => AccountId::unchecked_from(H256::from_slice(&address_raw)),
			_ => return Err(()),
		};

		Ok(account)
	}
}

impl pallet_cosmwasm::Config for Test {
	const MAX_FRAMES: u8 = 64;
	type RuntimeEvent = RuntimeEvent;
	type AccountIdExtended = AccountId;
	type PalletId = CosmwasmPalletId;
	type MaxCodeSize = ConstU32<{ 1024 * 1024 }>;
	type MaxInstrumentedCodeSize = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxMessageSize = ConstU32<{ 64 * 1024 }>;
	type AccountToAddr = AccountToAddr<Test>;
	type AssetToDenom = AssetToDenom;
	type Balance = Balance;
	type AssetId = AssetId;
	type Assets = Assets;
	type NativeAsset = Balances;
	type ChainId = ChainId;
	type MaxContractLabelSize = MaxContractLabelSize;
	type MaxContractTrieIdSize = MaxContractTrieIdSize;
	type MaxInstantiateSaltSize = MaxInstantiateSaltSize;
	type MaxFundsAssets = MaxFundsAssets;

	type CodeTableSizeLimit = CodeTableSizeLimit;
	type CodeGlobalVariableLimit = CodeGlobalVariableLimit;
	type CodeStackLimit = ConstU32<{ u32::MAX }>;

	type CodeParameterLimit = CodeParameterLimit;
	type CodeBranchTableSizeLimit = CodeBranchTableSizeLimit;
	type CodeStorageByteDeposit = CodeStorageByteDeposit;
	type ContractStorageByteReadPrice = ContractStorageByteReadPrice;
	type ContractStorageByteWritePrice = ContractStorageByteWritePrice;

	type WasmCostRules = WasmCostRules;
	type UnixTime = Timestamp;
	type WeightInfo = pallet_cosmwasm::weights::SubstrateWeight<Test>;

	// TODO: Add precompile to use execute or query pallet
	type PalletHook = ();

	type UploadWasmOrigin = frame_system::EnsureSigned<Self::AccountId>;

	type ExecuteWasmOrigin = frame_system::EnsureSigned<Self::AccountId>;
}
