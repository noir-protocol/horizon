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
	cosmos::bank::v1beta1::MsgSend,
	cosmwasm::wasm::v1::{
		MsgExecuteContract, MsgInstantiateContract2, MsgMigrateContract, MsgStoreCode,
		MsgUpdateAdmin,
	},
	Any,
};
use frame_support::{derive_impl, parameter_types, traits::AsEnsureOriginWithArg, PalletId};
use hp_account::CosmosSigner;
use hp_crypto::EcdsaExt;
use pallet_cosmos::{config_preludes::ChainId, AddressMapping};
use pallet_cosmos_types::msgservice::MsgHandler;
use pallet_cosmos_x_auth_signing::{
	any_match, sign_mode_handler::SignModeHandler, sign_verifiable_tx::SigVerifiableTx,
};
use pallet_cosmos_x_bank::msgs::MsgSendHandler;
use pallet_cosmos_x_wasm::msgs::{
	MsgExecuteContractHandler, MsgInstantiateContract2Handler, MsgMigrateContractHandler,
	MsgStoreCodeHandler, MsgUpdateAdminHandler,
};
use pallet_cosmwasm::instrument::CostRules;
use sp_core::{
	crypto::UncheckedFrom, ecdsa, ConstU128, ConstU32, ConstU64, Hasher, Pair, H160, H256,
};
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup, PostDispatchInfoOf},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type AssetId = u64;
type AccountId = CosmosSigner;
type Balance = u128;
type Hash = sp_core::H256;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Assets: pallet_assets,
		CosmosAccounts: pallet_cosmos_accounts,
		Cosmwasm: pallet_cosmwasm,
		Cosmos: pallet_cosmos,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
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

type AnteHandler<T> = pallet_cosmos_x_auth::AnteDecorators<T>;

#[derive_impl(pallet_cosmos::config_preludes::TestDefaultConfig)]
impl pallet_cosmos::Config for Test {
	type AddressMapping = HashedAddressMapping<Test, BlakeTwo256>;
	type NativeAsset = Balances;
	type Assets = Assets;
	type RuntimeEvent = RuntimeEvent;
	type AnteHandler = AnteHandler<Test>;
	type Balance = Balance;
	type AssetId = AssetId;
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

pub struct AccountToAddr<T>(PhantomData<T>);
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

impl pallet_cosmos_accounts::Config for Test {
	/// The overarching event type.
	type RuntimeEvent = RuntimeEvent;
	/// Weight information for extrinsics in this pallet.
	type WeightInfo = pallet_cosmos_accounts::weights::CosmosWeight<Test>;
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = AccountId;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Cosmos(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Cosmos(call) => match call.check_self_contained()? {
				Ok(address) => Some(Ok(
					<Test as pallet_cosmos::Config>::AddressMapping::from_address_raw(address),
				)),
				Err(e) => Some(Err(e)),
			},
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Cosmos(call) =>
				call.validate_self_contained(&info.to_cosmos_address().unwrap(), dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Cosmos(call) => call.pre_dispatch_self_contained(
				&info.to_cosmos_address().unwrap(),
				dispatch_info,
				len,
			),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Cosmos(pallet_cosmos::Call::transact { .. }) =>
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_cosmos::RawOrigin::CosmosTransaction(info.to_cosmos_address().unwrap()),
				))),
			_ => None,
		}
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let alice = CosmosSigner(ecdsa::Pair::from_string("//Alice", None).unwrap().public());
	let bob = CosmosSigner(ecdsa::Pair::from_string("//Bob", None).unwrap().public());

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(alice, 1_000_000_000_000_000_000), (bob, 1_000_000_000_000_000_000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_assets::GenesisConfig::<Test> {
		assets: vec![(0, alice, true, 1_000)],
		metadata: vec![(0, "stake".as_bytes().to_vec(), "stake".as_bytes().to_vec(), 18)],
		accounts: vec![(0, alice, 1_000_000_000_000_000_000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_cosmos_accounts::GenesisConfig::<Test> { accounts: vec![alice, bob] }
		.assimilate_storage(&mut t)
		.unwrap();

	sp_io::TestExternalities::new(t)
}
