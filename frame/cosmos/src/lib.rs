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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::comparison_chain, clippy::large_enum_variant)]

pub use self::pallet::*;
use cosmos_sdk_proto::prost::Message;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchInfo, PostDispatchInfo},
	pallet_prelude::{DispatchResultWithPostInfo, Pays},
	traits::{
		tokens::{fungible::Inspect, Fortitude, Preservation},
		Currency, Get,
	},
	weights::{constants::ExtrinsicBaseWeight, Weight},
};
use frame_system::{pallet_prelude::OriginFor, CheckWeight};
use pallet_cosmos_types::{
	handler::AnteDecorator,
	msgservice::MsgServiceRouter,
	tx::{Account, Gas},
};
use pallet_cosmos_x_auth_signing::{
	sign_mode_handler::SignModeHander, sign_verifiable_tx::SigVerifiableTx,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{
	traits::{BadOrigin, Convert, DispatchInfoOf, Dispatchable, UniqueSaturatedInto},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
	},
	DispatchError, RuntimeDebug,
};
use sp_std::{marker::PhantomData, vec::Vec};

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RawOrigin {
	CosmosTransaction(H160),
}

pub fn ensure_cosmos_transaction<OuterOrigin>(o: OuterOrigin) -> Result<H160, &'static str>
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(RawOrigin::CosmosTransaction(n)) => Ok(n),
		_ => Err("bad origin: expected to be a Cosmos transaction"),
	}
}

impl<T> Call<T>
where
	OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	T: Send + Sync + Config,
	T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	pub fn is_self_contained(&self) -> bool {
		matches!(self, Call::transact { .. })
	}

	pub fn check_self_contained(&self) -> Option<Result<H160, TransactionValidityError>> {
		None
	}

	pub fn pre_dispatch_self_contained(
		&self,
		origin: &H160,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		if let Call::transact { tx_bytes } = self {
			if let Err(e) = CheckWeight::<T>::do_pre_dispatch(dispatch_info, len) {
				return Some(Err(e));
			}

			Some(Pallet::<T>::validate_transaction_in_block(*origin, tx_bytes))
		} else {
			None
		}
	}

	pub fn validate_self_contained(
		&self,
		origin: &H160,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		if let Call::transact { tx_bytes } = self {
			if let Err(e) = CheckWeight::<T>::do_validate(dispatch_info, len) {
				return Some(Err(e));
			}

			Some(Pallet::<T>::validate_transaction_in_pool(*origin, tx_bytes))
		} else {
			None
		}
	}
}

pub trait AddressMapping<A> {
	fn into_account_id(address: H160) -> A;
}

pub trait EnsureAddressOrigin<OuterOrigin> {
	/// Success return type.
	type Success;

	/// Perform the origin check.
	fn ensure_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<Self::Success, BadOrigin> {
		Self::try_address_origin(address, origin).map_err(|_| BadOrigin)
	}

	/// Try with origin.
	fn try_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<Self::Success, OuterOrigin>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::Contains};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::origin]
	pub type Origin = RawOrigin;

	/// Type alias for currency balance.
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Default implementations of [`DefaultConfig`], which can be used to implement [`Config`].
	pub mod config_preludes {
		use super::*;
		use frame_support::{derive_impl, parameter_types};
		pub struct TestDefaultConfig;

		#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig, no_aggregated_types)]
		impl frame_system::DefaultConfig for TestDefaultConfig {}

		pub struct MsgFilter;
		impl Contains<Vec<u8>> for MsgFilter {
			fn contains(_type_url: &Vec<u8>) -> bool {
				true
			}
		}

		pub struct GasToWeight;
		impl Convert<Gas, Weight> for GasToWeight {
			fn convert(gas: Gas) -> Weight {
				Weight::from_parts(gas, 0u64)
			}
		}

		pub struct WeightToGas;
		impl Convert<Weight, Gas> for WeightToGas {
			fn convert(weight: Weight) -> Gas {
				weight.ref_time()
			}
		}

		parameter_types! {
			pub const MaxMemoCharacters: u64 = 256;
			pub const StringLimit: u32 = 128;
			pub NativeDenom: BoundedVec<u8, StringLimit> = (*b"acdt").to_vec().try_into().unwrap();
			pub ChainId: BoundedVec<u8, StringLimit> = (*b"dev").to_vec().try_into().unwrap();
			pub const TxSigLimit: u64 = 7;
		}

		#[frame_support::register_default_impl(TestDefaultConfig)]
		impl DefaultConfig for TestDefaultConfig {
			#[inject_runtime_type]
			type RuntimeEvent = ();
			type AnteHandler = ();
			type MaxMemoCharacters = MaxMemoCharacters;
			type NativeDenom = NativeDenom;
			type StringLimit = StringLimit;
			type MsgServiceRouter = ();
			type ChainId = ChainId;
			type MsgFilter = MsgFilter;
			type GasToWeight = GasToWeight;
			type WeightToGas = WeightToGas;
			type TxSigLimit = TxSigLimit;
			type SigVerifiableTx = ();
			type SignModeHander = ();
		}
	}

	#[pallet::config(with_default)]
	pub trait Config: frame_system::Config {
		/// Mapping from address to account id.
		#[pallet::no_default]
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// Currency type for withdraw and balance storage.
		#[pallet::no_default]
		type Currency: Currency<Self::AccountId> + Inspect<Self::AccountId>;
		/// The overarching event type.
		#[pallet::no_default_bounds]
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Verify the validity of a Cosmos transaction.
		type AnteHandler: AnteDecorator;
		/// The maximum size of the memo.
		#[pallet::constant]
		type MaxMemoCharacters: Get<u64>;
		/// The native denomination for the currency.
		#[pallet::constant]
		type NativeDenom: Get<BoundedVec<u8, Self::StringLimit>>;
		/// The maximum length of string value.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// Router for message service handling.
		type MsgServiceRouter: MsgServiceRouter;
		/// The chain ID.
		#[pallet::constant]
		type ChainId: Get<BoundedVec<u8, Self::StringLimit>>;
		/// The message filter.
		type MsgFilter: Contains<Vec<u8>>;
		/// The converter for converting Gas to Weight.
		type GasToWeight: Convert<Gas, Weight>;
		/// The converter for converting Weight to Gas.
		type WeightToGas: Convert<Weight, Gas>;
		/// The maximum number of transaction signatures allowed.
		#[pallet::constant]
		type TxSigLimit: Get<u64>;

		type SigVerifiableTx: SigVerifiableTx;

		type SignModeHander: SignModeHander;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event {
		Executed(pallet_cosmos_types::events::Event),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTx,
		Unauthorized,
		InsufficientFunds,
		OutOfGas,
		InsufficientFee,
		InvalidType,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Transact a Cosmos transaction.
		#[pallet::call_index(0)]
		#[pallet::weight({ 0 })]
		pub fn transact(origin: OriginFor<T>, tx_bytes: Vec<u8>) -> DispatchResultWithPostInfo {
			let source = ensure_cosmos_transaction(origin)?;
			let tx = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx::decode(&mut &*tx_bytes).unwrap();

			Self::apply_validated_transaction(source, tx)
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Validate a Cosmos transaction already in block
	///
	/// This function must be called during the pre-dispatch phase
	/// (just before applying the extrinsic).
	pub fn validate_transaction_in_block(
		origin: H160,
		tx_bytes: &[u8],
	) -> Result<(), TransactionValidityError> {
		let (_who, _) = Self::account(&origin);
		let tx = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx::decode(&mut &*tx_bytes).unwrap();

		T::AnteHandler::ante_handle(&tx, false)?;

		Ok(())
	}

	// Controls that must be performed by the pool.
	fn validate_transaction_in_pool(origin: H160, tx_bytes: &[u8]) -> TransactionValidity {
		// let (who, _) = Self::account(&origin);
		// let tx = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx::decode(tx_bytes).unwrap();

		// T::AnteHandler::ante_handle(&tx, true)?;

		// let transaction_nonce = tx
		// 	.auth_info
		// 	.signer_infos
		// 	.first()
		// 	.ok_or(TransactionValidityError::Invalid(InvalidTransaction::Call))?
		// 	.sequence;

		// let mut builder =
		// 	ValidTransactionBuilder::default().and_provides((origin, transaction_nonce));

		// // In the context of the pool, a transaction with
		// // too high a nonce is still considered valid
		// if transaction_nonce > who.sequence {
		// 	if let Some(prev_nonce) = transaction_nonce.checked_sub(1) {
		// 		builder = builder.and_requires((origin, prev_nonce))
		// 	}
		// }

		// builder.build()
		Ok(Default::default())
	}

	fn apply_validated_transaction(
		source: H160,
		tx: cosmos_sdk_proto::cosmos::tx::v1beta1::Tx,
	) -> DispatchResultWithPostInfo {
		let mut total_weight = ExtrinsicBaseWeight::get();

		if let Some(body) = tx.body {
			for msg in body.messages.iter() {
				let handler =
					T::MsgServiceRouter::route(&msg.type_url).ok_or(DispatchErrorWithPostInfo {
						post_info: PostDispatchInfo {
							actual_weight: Some(total_weight),
							pays_fee: Pays::Yes,
						},
						error: DispatchError::Other("Unknown message type"),
					})?;
				match handler.handle(msg) {
					Ok(weight) => {
						total_weight = total_weight.saturating_add(weight);
					},
					Err(e) => {
						total_weight = total_weight.saturating_add(e.weight);

						return Err(DispatchErrorWithPostInfo {
							post_info: PostDispatchInfo {
								actual_weight: Some(total_weight),
								pays_fee: Pays::Yes,
							},
							error: DispatchError::Other("Failed to handle message"),
						});
					},
				}
			}
		} else {
			return Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(total_weight),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("Invalid tx"),
			});
		}

		Ok(PostDispatchInfo { actual_weight: Some(total_weight), pays_fee: Pays::Yes })
	}

	/// Get the base account info.
	pub fn account(address: &H160) -> (Account, frame_support::weights::Weight) {
		let account_id = T::AddressMapping::into_account_id(*address);

		let nonce = frame_system::Pallet::<T>::account_nonce(&account_id);
		// keepalive `true` takes into account ExistentialDeposit as part of what's considered
		// liquid balance.
		let balance =
			T::Currency::reducible_balance(&account_id, Preservation::Preserve, Fortitude::Polite);

		(
			Account {
				sequence: UniqueSaturatedInto::<u64>::unique_saturated_into(nonce),
				amount: UniqueSaturatedInto::<u128>::unique_saturated_into(balance),
			},
			T::DbWeight::get().reads(2),
		)
	}
}
