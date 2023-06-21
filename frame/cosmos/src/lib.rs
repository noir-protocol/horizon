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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::comparison_chain, clippy::large_enum_variant)]
#![deny(unused_crate_dependencies)]

mod runner;

use crate::runner::{Runner as RunnerT, RunnerError};
use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	dispatch::{DispatchInfo, PostDispatchInfo},
	pallet_prelude::DispatchResult,
	scale_info::TypeInfo,
	traits::{tokens::fungible::Inspect, Currency, ExistenceRequirement},
	weights::Weight,
};
use frame_system::{pallet_prelude::OriginFor, CheckWeight};
use primitive_types::{H160, U128};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
	},
	RuntimeDebug,
};
use sp_std::marker::PhantomData;

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
		_ => Err("bad origin: expected to be an Cosmos transaction"),
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
		if let Call::transact { tx } = self {
			let check = || {
				let origin = Pallet::<T>::verify(tx).ok_or(
					// TODO: Define error code
					InvalidTransaction::Custom(0u8),
				)?;

				Ok(origin)
			};

			Some(check())
		} else {
			None
		}
	}

	pub fn pre_dispatch_self_contained(
		&self,
		origin: &H160,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		if let Call::transact { tx } = self {
			if let Err(e) = CheckWeight::<T>::do_pre_dispatch(dispatch_info, len) {
				return Some(Err(e))
			}

			Some(Pallet::<T>::validate_transaction_in_block(*origin, tx))
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
		if let Call::transact { tx } = self {
			if let Err(e) = CheckWeight::<T>::do_validate(dispatch_info, len) {
				return Some(Err(e))
			}

			Some(Pallet::<T>::validate_transaction_in_pool(*origin, tx))
		} else {
			None
		}
	}
}

pub use self::pallet::*;

pub trait AddressMapping<A> {
	fn into_account_id(address: H160) -> A;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance to perform action
		BalanceLow,
		/// Invalid amount type
		InvalidAmount,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Mapping from address to account id.
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// Currency type for withdraw and balance storage.
		type Currency: Currency<Self::AccountId> + Inspect<Self::AccountId>;
		/// Cosmos execution runner.
		type Runner: RunnerT<Self>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Transact an Cosmos transaction.
		#[pallet::call_index(0)]
		#[pallet::weight({ tx.auth_info.fee.gas_limit })]
		pub fn transact(origin: OriginFor<T>, tx: hp_cosmos::Tx) -> DispatchResult {
			let source = ensure_cosmos_transaction(origin)?;
			Self::apply_validated_transaction(source, tx)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn verify(tx: &hp_cosmos::Tx) -> Option<H160> {
		if let Some(public_key) = &tx.auth_info.signer_infos[0].public_key {
			match public_key {
				hp_cosmos::SignerPublicKey::Single(hp_cosmos::PublicKey::SECP256K1(pk)) =>
					if hp_io::crypto::secp256k1_ecdsa_verify(&pk, &tx.hash, &tx.signatures[0][..]) {
						Some(hp_io::crypto::ripemd160(&sp_io::hashing::sha2_256(pk)).into())
					} else {
						None
					},
				_ => None,
			}
		} else {
			None
		}
	}

	/// Validate an Cosmos transaction already in block
	///
	/// This function must be called during the pre-dispatch phase
	/// (just before applying the extrinsic).
	pub fn validate_transaction_in_block(
		origin: H160,
		tx: &hp_cosmos::Tx,
	) -> Result<(), TransactionValidityError> {
		// TODO: Check nonce and fee
		Ok(())
	}

	// Controls that must be performed by the pool.
	// The controls common with the State Transition Function (STF) are in
	// the function `validate_transaction_common`.
	fn validate_transaction_in_pool(origin: H160, tx: &hp_cosmos::Tx) -> TransactionValidity {
		// TODO: Check nonce and fee
		// TODO: Build transaction with priority and nonce

		// let mut builder = ValidTransactionBuilder::default()
		// 	.and_provides((origin, transaction_nonce))
		// 	.priority(priority);

		// In the context of the pool, a transaction with
		// too high a nonce is still considered valid
		// if transaction_nonce > who.nonce {
		// 	if let Some(prev_nonce) = transaction_nonce.checked_sub(1.into()) {
		// 		builder = builder.and_requires((origin, prev_nonce))
		// 	}
		// }

		// builder.build()
		ValidTransactionBuilder::default().into()
	}

	fn apply_validated_transaction(source: H160, tx: hp_cosmos::Tx) -> DispatchResult {
		match tx.body.messages[0].clone() {
			hp_cosmos::Message::MsgSend { from_address, to_address, amount } => {
				T::Runner::msg_send(from_address, to_address, amount[0].amount.into())
					.map_err(|_| Error::<T>::BalanceLow)?;
			},
		};
		Ok(())
	}
}

#[derive(Default)]
pub struct Runner<T: Config> {
	_marker: PhantomData<T>,
}

/// Type alias for currency balance.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

impl<T: Config> RunnerT<T> for Runner<T>
where
	BalanceOf<T>: TryFrom<U128> + Into<U128>,
{
	type Error = Error<T>;

	fn msg_send(
		from_address: H160,
		to_address: H160,
		amount: U128,
	) -> Result<(), RunnerError<Self::Error>> {
		let source = T::AddressMapping::into_account_id(from_address);
		let target = T::AddressMapping::into_account_id(to_address);
		let value = amount.try_into().map_err(|_| RunnerError {
			error: Self::Error::InvalidAmount,
			weight: Weight::default(),
		})?;
		T::Currency::transfer(&source, &target, value, ExistenceRequirement::AllowDeath);
		Ok(())
	}
}
