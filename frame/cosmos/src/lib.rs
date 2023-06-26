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

pub mod runner;

use crate::runner::{Runner as RunnerT, RunnerError};
use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	dispatch::{DispatchInfo, PostDispatchInfo},
	pallet_prelude::DispatchResult,
	scale_info::TypeInfo,
	traits::{tokens::fungible::Inspect, Currency, ExistenceRequirement, Get},
	weights::Weight,
};
use frame_system::{pallet_prelude::OriginFor, CheckWeight};
use hp_cosmos::Account;
use sp_core::H160;
use sp_runtime::{
	traits::{BadOrigin, DispatchInfoOf, Dispatchable, UniqueSaturatedInto},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
	},
	DispatchError, RuntimeDebug,
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
		matches!(self, Call::broadcast_tx { .. })
	}

	pub fn check_self_contained(&self) -> Option<Result<H160, TransactionValidityError>> {
		if let Call::broadcast_tx { tx } = self {
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
		if let Call::broadcast_tx { tx } = self {
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
		if let Call::broadcast_tx { tx } = self {
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

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance to perform action
		BalanceLow,
		/// Invalid type
		InvalidType,
		/// Unauthorized access
		UnauthorizedAccess,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::origin]
	pub type Origin = RawOrigin;

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
		pub fn broadcast_tx(origin: OriginFor<T>, tx: hp_cosmos::Tx) -> DispatchResult {
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
					if hp_io::crypto::secp256k1_ecdsa_verify(&pk, &tx.hash, &tx.signatures[0]) {
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
		let (who, _) = Self::account(&origin);
		if who.sequence != tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
		}
		Ok(())
	}

	// Controls that must be performed by the pool.
	// The controls common with the State Transition Function (STF) are in
	// the function `validate_transaction_common`.
	fn validate_transaction_in_pool(origin: H160, tx: &hp_cosmos::Tx) -> TransactionValidity {
		let (who, _) = Self::account(&origin);
		if who.sequence != tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
		}

		let transaction_nonce = tx.auth_info.signer_infos[0].sequence;
		let mut builder =
			ValidTransactionBuilder::default().and_provides((origin, transaction_nonce));

		// In the context of the pool, a transaction with
		// too high a nonce is still considered valid
		if transaction_nonce > who.sequence {
			if let Some(prev_nonce) = transaction_nonce.checked_sub(1) {
				builder = builder.and_requires((origin, prev_nonce))
			}
		}

		builder.build()
	}

	fn apply_validated_transaction(source: H160, tx: hp_cosmos::Tx) -> DispatchResult {
		match tx.body.messages[0].clone() {
			hp_cosmos::Message::MsgSend { from_address, to_address, amount } => {
				if source != from_address {
					return Err(DispatchError::from(Error::<T>::UnauthorizedAccess))
				}
				T::Runner::msg_send(from_address, to_address, amount.into())
					.map_err(|_| Error::<T>::BalanceLow)?;
			},
		};
		Ok(())
	}

	/// Get the base account info.
	pub fn account(address: &H160) -> (Account, frame_support::weights::Weight) {
		let account_id = T::AddressMapping::into_account_id(*address);

		let nonce = frame_system::Pallet::<T>::account_nonce(&account_id);
		// keepalive `true` takes into account ExistentialDeposit as part of what's considered
		// liquid balance.
		let balance = T::Currency::reducible_balance(&account_id, true);

		(
			Account {
				sequence: UniqueSaturatedInto::<u64>::unique_saturated_into(nonce),
				amount: UniqueSaturatedInto::<u128>::unique_saturated_into(balance),
			},
			T::DbWeight::get().reads(2),
		)
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
	BalanceOf<T>: TryFrom<u128> + Into<u128>,
{
	type Error = Error<T>;

	fn msg_send(
		from_address: H160,
		to_address: H160,
		amount: u128,
	) -> Result<(), RunnerError<Self::Error>> {
		let source = T::AddressMapping::into_account_id(from_address);
		let target = T::AddressMapping::into_account_id(to_address);
		let value = amount.try_into().map_err(|_| RunnerError {
			error: Self::Error::InvalidType,
			weight: Weight::default(),
		})?;
		T::Currency::transfer(&source, &target, value, ExistenceRequirement::AllowDeath).map_err(
			|_| RunnerError { error: Self::Error::BalanceLow, weight: Weight::default() },
		)?;
		Ok(())
	}
}
