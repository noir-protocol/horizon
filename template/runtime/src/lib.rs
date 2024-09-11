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

//! Horizon runtime.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

mod accounts;
mod ante;
mod assets;
mod compat;
mod msgs;

use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::marker::PhantomData;
use cosmos_runtime_api::{GasInfo, SimulateError, SimulateResponse, SimulateResult};
use cosmos_sdk_proto::{cosmos::tx::v1beta1::Tx, prost::Message};
use frame_support::{
	construct_runtime, derive_impl,
	genesis_builder_helper::{build_config, create_default_config},
	pallet_prelude::InvalidTransaction,
	parameter_types,
	traits::{
		tokens::{fungible, Fortitude, Preservation},
		AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU32, ConstU8, OnTimestampSet,
	},
	weights::{
		constants::{RocksDbWeight as RuntimeDbWeight, WEIGHT_REF_TIME_PER_MILLIS},
		IdentityFee, Weight,
	},
	PalletId,
};
use frame_system::EnsureRoot;
use hp_account::CosmosSigner;
use hp_crypto::EcdsaExt;
use pallet_cosmos::{
	config_preludes::{
		AddressPrefix, ChainId, MaxDenomLimit, MaxMemoCharacters, MsgFilter, NativeDenom,
		TxSigLimit, WeightToGas,
	},
	AddressMapping,
};
use pallet_cosmos_types::{address::acc_address_from_bech32, any_match, context::Context};
use pallet_cosmos_x_auth_signing::{
	sign_mode_handler::SignModeHandler, sign_verifiable_tx::SigVerifiableTx,
};
use pallet_cosmwasm::instrument::CostRules;
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, ecdsa::Public, OpaqueMetadata, H160};
use sp_runtime::{
	codec, create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, DispatchInfoOf, Dispatchable,
		IdentifyAccount, NumberFor, One, PostDispatchInfoOf, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
	ApplyExtrinsicResult, ExtrinsicInclusionMode, Perbill,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

/// Type of block number.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = hp_account::CosmosSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("horizon-template"),
	impl_name: create_runtime_str!("horizon-template"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// Native version.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

pub const MILLISECS_PER_BLOCK: u64 = 6000;
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2000ms of compute with a 6 second average block time.
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 2000;
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_MILLISECS_PER_BLOCK * WEIGHT_REF_TIME_PER_MILLIS, u64::MAX);
pub const MAXIMUM_BLOCK_LENGTH: u32 = 5 * 1024 * 1024;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 256;
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(MAXIMUM_BLOCK_LENGTH, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The block type.
	type Block = Block;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RuntimeDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

type AssetId = u128;

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetIdParameter = codec::Compact<AssetId>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = ConstU128<500>;
	type AssetAccountDeposit = ConstU128<500>;
	type MetadataDepositBase = ConstU128<0>;
	type MetadataDepositPerByte = ConstU128<0>;
	type ApprovalDeposit = ConstU128<0>;
	type StringLimit = ConstU32<20>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = assets::AssetsCallback<Runtime>;
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
}

parameter_types! {
	pub const MaxAuthorities: u32 = 100;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = MaxAuthorities;
	type DisabledValidators = ();
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ();

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// Handler for withdrawing, refunding and depositing the transaction fee.
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	/// A fee multiplier for `Operational` extrinsics to compute "virtual tip" to boost their
	/// `priority`.
	type OperationalFeeMultiplier = ConstU8<5>;
	/// Convert a weight value into a deductible fee based on the currency type.
	type WeightToFee = IdentityFee<Balance>;
	/// Convert a length value into a deductible fee based on the currency type.
	type LengthToFee = IdentityFee<Balance>;
	/// Update the multiplier of the next block, based on the previous block's weight.
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
	pub storage EnableManualSeal: bool = false;
}

pub struct ConsensusOnTimestampSet<T>(PhantomData<T>);
impl<T: pallet_aura::Config> OnTimestampSet<T::Moment> for ConsensusOnTimestampSet<T> {
	fn on_timestamp_set(moment: T::Moment) {
		if EnableManualSeal::get() {
			return;
		}
		<pallet_aura::Pallet<T> as OnTimestampSet<T::Moment>>::on_timestamp_set(moment)
	}
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ConsensusOnTimestampSet<Self>;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_cosmos::Config for Runtime {
	/// Mapping an address to an account id.
	type AddressMapping = compat::cosmos::HashedAddressMapping<Self, BlakeTwo256>;
	/// Native asset type.
	type NativeAsset = Balances;
	/// Type of an account balance.
	type Balance = Balance;
	/// Type of a tradable asset id.
	/// The [`Ord`] constraint is required for [`BoundedBTreeMap`].
	type AssetId = AssetId;
	/// Interface from which we are going to execute assets operations.
	type Assets = Assets;
	/// The overarching event type.
	type RuntimeEvent = RuntimeEvent;
	/// Verify the validity of a Cosmos transaction.
	type AnteHandler = ante::AnteHandler<Self>;
	/// The maximum number of characters allowed in a memo.
	type MaxMemoCharacters = MaxMemoCharacters;
	/// The native denomination for the currency.
	type NativeDenom = NativeDenom;
	/// Router for handling message services.
	type MsgServiceRouter = msgs::MsgServiceRouter<Self>;
	/// The chain ID.
	type ChainId = ChainId;
	/// The message filter.
	type MsgFilter = MsgFilter;
	/// Converts Weight to Gas and Gas to Weight.
	type WeightToGas = WeightToGas;
	/// The maximum number of transaction signatures allowed.
	type TxSigLimit = TxSigLimit;
	/// Defines the features for all signature verification handlers.
	type SigVerifiableTx = SigVerifiableTx;
	/// Handler for managing different signature modes in transactions.
	type SignModeHandler = SignModeHandler;

	type WeightInfo = pallet_cosmos::weights::CosmosWeight<Runtime>;

	type AssetToDenom = assets::AssetToDenom<Runtime>;

	type MaxDenomLimit = MaxDenomLimit;

	type AddressPrefix = AddressPrefix;

	type Context = Context;
}

impl pallet_cosmos_accounts::Config for Runtime {
	/// The overarching event type.
	type RuntimeEvent = RuntimeEvent;
	/// Weight information for extrinsics in this pallet.
	type WeightInfo = pallet_cosmos_accounts::weights::CosmosWeight<Runtime>;
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
	pub WasmCostRules: CostRules<Runtime> = Default::default();
}

impl pallet_cosmwasm::Config for Runtime {
	const MAX_FRAMES: u8 = 64;
	type RuntimeEvent = RuntimeEvent;
	type AccountIdExtended = AccountId;
	type PalletId = CosmwasmPalletId;
	type MaxCodeSize = ConstU32<{ 1024 * 1024 }>;
	type MaxInstrumentedCodeSize = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxMessageSize = ConstU32<{ 64 * 1024 }>;
	type AccountToAddr = accounts::AccountToAddr<Runtime>;
	type AssetToDenom = assets::AssetToDenom<Runtime>;
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
	type WeightInfo = pallet_cosmwasm::weights::SubstrateWeight<Runtime>;

	// TODO: Add precompile to use execute or query pallet
	type PalletHook = ();

	type UploadWasmOrigin = frame_system::EnsureSigned<Self::AccountId>;

	type ExecuteWasmOrigin = frame_system::EnsureSigned<Self::AccountId>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Self>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Aura: pallet_aura,
		Cosmos: pallet_cosmos,
		CosmosAccounts: pallet_cosmos_accounts,
		Cosmwasm: pallet_cosmwasm,
		Grandpa: pallet_grandpa,
		Sudo: pallet_sudo,
		Timestamp: pallet_timestamp,
		TransactionPayment: pallet_transaction_payment,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

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
					<Runtime as pallet_cosmos::Config>::AddressMapping::into_account_id(address),
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
			RuntimeCall::Cosmos(call) => {
				if let pallet_cosmos::Call::transact { tx_bytes } = call {
					if Runtime::migrate_cosm_account(tx_bytes).is_err() {
						return Some(Err(TransactionValidityError::Invalid(
							InvalidTransaction::BadSigner,
						)));
					}
				}

				call.validate_self_contained(&info.to_cosmos_address().unwrap(), dispatch_info, len)
			},
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
			RuntimeCall::Cosmos(call) => {
				if let pallet_cosmos::Call::transact { tx_bytes } = call {
					if Runtime::migrate_cosm_account(tx_bytes).is_err() {
						return Some(Err(TransactionValidityError::Invalid(
							InvalidTransaction::BadSigner,
						)));
					}
				}

				call.pre_dispatch_self_contained(
					&info.to_cosmos_address().unwrap(),
					dispatch_info,
					len,
				)
			},
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

impl Runtime {
	fn migrate_cosm_account(tx_bytes: &[u8]) -> Result<(), TransactionValidityError> {
		use cosmos_sdk_proto::cosmos::crypto::secp256k1;
		use fungible::{Inspect, Mutate};
		use pallet_cosmos_x_auth_signing::sign_verifiable_tx::traits::SigVerifiableTx;

		let tx = Tx::decode(&mut &*tx_bytes)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;
		let signers = <Runtime as pallet_cosmos::Config>::SigVerifiableTx::get_signers(&tx)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

		let signer_infos = tx
			.auth_info
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?
			.signer_infos;

		for (i, signer_info) in signer_infos.iter().enumerate() {
			if signer_info.sequence == 0 {
				let signer = signers
					.get(i)
					.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

				let (_hrp, address_raw) = acc_address_from_bech32(signer).map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
				})?;
				if address_raw.len() != 20 {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
				}
				let interim_account =
					<Runtime as pallet_cosmos::Config>::AddressMapping::into_account_id(
						H160::from_slice(&address_raw),
					);

				let public_key = signer_info
					.public_key
					.as_ref()
					.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?;
				let who = any_match!(
					public_key, {
						secp256k1::PubKey => {
							let public_key = secp256k1::PubKey::decode(&mut &*public_key.value)
								.map_err(|_| {
									TransactionValidityError::Invalid(InvalidTransaction::Call)
								})?;
							let mut pk = [0u8; 33];
							pk.copy_from_slice(&public_key.key);

							Ok(CosmosSigner(Public(pk)))
						}
					},
					Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
				)?;

				let balance = pallet_balances::Pallet::<Runtime>::reducible_balance(
					&interim_account,
					Preservation::Expendable,
					Fortitude::Polite,
				);
				<pallet_balances::Pallet<Runtime> as Mutate<AccountId>>::transfer(
					&interim_account,
					&who,
					balance,
					Preservation::Expendable,
				)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

				// TODO: Add asset transfer for migration

				pallet_cosmos_accounts::Pallet::<Runtime>::connect_account(&who)
					.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;
			}
		}

		Ok(())
	}
}

impl_runtime_apis! {
	impl cosmos_runtime_api::CosmosRuntimeApi<Block> for Runtime {
		fn convert_tx(tx_bytes: Vec<u8>) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_cosmos::Call::<Runtime>::transact { tx_bytes }.into(),
			)
		}

		fn simulate(tx_bytes: Vec<u8>) -> SimulateResult {
			let tx = Tx::decode(&mut &*tx_bytes).map_err(|_| SimulateError::InvalidTx)?;

			// TODO: Run ante handlers

			pallet_cosmos::Pallet::<Runtime>::apply_validated_transaction(H160::default(), tx.clone())
				.map_err(|e| SimulateError::InternalError(format!("Failed to simulate cosmos tx. error: {:?}", e).into()))?;

			System::read_events_no_consensus()
				.find_map(|record| {
					if let RuntimeEvent::Cosmos(pallet_cosmos::Event::Executed { gas_wanted, gas_used, events }) = record.event {
						Some(SimulateResponse{gas_info: GasInfo { gas_wanted, gas_used }, events})
					} else {
						None
					}
				}).ok_or(SimulateError::InternalError("Cosmos events does not exist".into()))
		}
	}

	impl cosmwasm_runtime_api::CosmwasmRuntimeApi<Block, Vec<u8>> for Runtime {
		fn query(
			contract: String,
			gas: u64,
			query_request: Vec<u8>,
		) -> Result<Vec<u8>, Vec<u8>>{
			let contract = <Runtime as pallet_cosmwasm::Config>::AccountToAddr::convert(contract).map_err(|_| "Invalid contract address".as_bytes().to_vec())?;
			match pallet_cosmwasm::query::<Runtime>(
				contract,
				gas,
				query_request,
			) {
				Ok(response) => Ok(response.into()),
				Err(err) => Err(alloc::format!("{:?}", err).into_bytes())
			}
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().to_vec()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}

		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}

		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}
}
