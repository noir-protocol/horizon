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
#![deny(unused_crate_dependencies)]

pub mod runner;
pub mod weights;

use crate::runner::{Runner as RunnerT, RunnerError};
use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	dispatch::{DispatchInfo, Pays, PostDispatchInfo},
	pallet_prelude::{DispatchClass, DispatchResultWithPostInfo},
	scale_info::TypeInfo,
	traits::{tokens::fungible::Inspect, Currency, ExistenceRequirement, Get, WithdrawReasons},
	weights::{Weight, WeightToFee},
};
use frame_system::{pallet_prelude::OriginFor, CheckWeight};
use hp_cosmos::{Account, AuthInfo, Msg};
pub use pallet::*;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{BadOrigin, Convert, DispatchInfoOf, Dispatchable, UniqueSaturatedInto},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransactionBuilder,
	},
	DispatchErrorWithPostInfo, RuntimeDebug,
};
use sp_std::{marker::PhantomData, vec::Vec};
pub use weights::*;

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
				let origin = Pallet::<T>::verify(tx).ok_or(InvalidTransaction::Custom(
					fp_ethereum::TransactionValidationError::InvalidSignature as u8,
				))?;

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
	use frame_support::pallet_prelude::*;

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance to perform action
		BalanceLow,
		/// Invalid type
		InvalidType,
		/// Unauthorized access
		UnauthorizedAccess,
		/// Calculating total fee overflowed
		FeeOverflow,
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
		/// Convert a length value into a deductible fee based on the currency type.
		type LengthToFee: WeightToFee<Balance = BalanceOf<Self>>;
		/// Cosmos execution runner.
		type Runner: RunnerT<Self>;
		/// The overarching event type.
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
		/// Used to answer contracts' queries regarding the current weight price. This is **not**
		/// used to calculate the actual fee and is only for informational purposes.
		type WeightPrice: Convert<Weight, BalanceOf<Self>>;
		/// Convert a weight value into a deductible fee based on the currency type.
		type WeightToFee: WeightToFee<Balance = BalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		Executed {
			tx_hash: H256,
			code: u8,
			gas_used: Option<Weight>,
			auth_info: AuthInfo,
			messages: Vec<Msg>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Transact an Cosmos transaction.
		#[pallet::call_index(0)]
		#[pallet::weight(tx.auth_info.fee.gas_limit)]
		pub fn transact(origin: OriginFor<T>, tx: hp_cosmos::Tx) -> DispatchResultWithPostInfo {
			let source = ensure_cosmos_transaction(origin)?;
			Self::apply_validated_transaction(source, tx)
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn verify(tx: &hp_cosmos::Tx) -> Option<H160> {
		if let Some(public_key) = &tx.auth_info.signer_infos[0].public_key {
			match public_key {
				hp_cosmos::SignerPublicKey::Single(hp_cosmos::PublicKey::Secp256k1(pk)) =>
					if hp_io::crypto::secp256k1_ecdsa_verify(
						&tx.signatures[0],
						tx.hash.as_bytes(),
						&pk[..],
					) {
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
		if who.sequence < tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
		}
		if who.sequence > tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Future))
		}

		let mut total_payment = 0u128;
		total_payment = total_payment.saturating_add(tx.auth_info.fee.amount);
		for msg in tx.body.messages.iter() {
			match msg {
				Msg::MsgSend { amount, .. } => {
					total_payment = total_payment.saturating_add(*amount);
				},
			}
		}
		if total_payment > who.amount {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Payment))
		}

		Ok(())
	}

	// Controls that must be performed by the pool.
	fn validate_transaction_in_pool(origin: H160, tx: &hp_cosmos::Tx) -> TransactionValidity {
		let (who, _) = Self::account(&origin);
		if who.sequence < tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale))
		}
		if who.sequence > tx.auth_info.signer_infos[0].sequence {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Future))
		}
		let mut total_payment = 0u128;
		total_payment = total_payment.saturating_add(tx.auth_info.fee.amount);
		for msg in tx.body.messages.iter() {
			match msg {
				Msg::MsgSend { amount, .. } => {
					total_payment = total_payment.saturating_add(*amount);
				},
			}
		}
		if total_payment > who.amount {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::Payment))
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

	fn apply_validated_transaction(source: H160, tx: hp_cosmos::Tx) -> DispatchResultWithPostInfo {
		match Self::execute(&source, &tx) {
			Ok(weight) => {
				Self::deposit_event(Event::Executed {
					tx_hash: tx.hash.into(),
					code: 0u8,
					gas_used: Some(weight),
					auth_info: tx.auth_info.clone(),
					messages: tx.body.messages.clone(),
				});
				Ok(PostDispatchInfo { actual_weight: Some(weight), pays_fee: Pays::No })
			},
			Err(e) => {
				Self::deposit_event(Event::Executed {
					tx_hash: tx.hash.into(),
					code: 1u8,
					gas_used: e.post_info.actual_weight,
					auth_info: tx.auth_info.clone(),
					messages: tx.body.messages.clone(),
				});
				Err(e)
			},
		}
	}

	fn execute(
		source: &H160,
		tx: &hp_cosmos::Tx,
	) -> Result<Weight, DispatchErrorWithPostInfo<PostDispatchInfo>> {
		let mut total_weight = Weight::default();
		for msg in tx.body.messages.iter() {
			match msg {
				hp_cosmos::Msg::MsgSend { from_address, to_address, amount } => {
					if *source != *from_address {
						return Err(DispatchErrorWithPostInfo {
							post_info: PostDispatchInfo {
								actual_weight: Some(total_weight),
								pays_fee: Pays::Yes,
							},
							error: Error::<T>::UnauthorizedAccess.into(),
						})
					}
					let weight =
						T::Runner::msg_send(from_address, to_address, *amount).map_err(|e| {
							DispatchErrorWithPostInfo {
								post_info: PostDispatchInfo {
									actual_weight: Some(total_weight.saturating_add(e.weight)),
									pays_fee: Pays::Yes,
								},
								error: e.error.into(),
							}
						})?;
					total_weight = total_weight.saturating_add(weight);
				},
			};
		}
		// Add account nonce increment weight.
		total_weight = total_weight.saturating_add(
			T::DbWeight::get().reads(1).saturating_add(T::DbWeight::get().writes(1)),
		);
		let fee = Self::compute_fee(tx.len, total_weight);
		let maximum_fee = tx.auth_info.fee.amount.unique_saturated_into();
		if fee > maximum_fee {
			return Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(total_weight),
					pays_fee: Pays::Yes,
				},
				error: Error::<T>::FeeOverflow.into(),
			})
		}
		let source = T::AddressMapping::into_account_id(*source);
		T::Currency::withdraw(&source, fee, WithdrawReasons::FEE, ExistenceRequirement::AllowDeath)
			.map_err(|e| DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(total_weight),
					pays_fee: Pays::Yes,
				},
				error: e,
			})?;
		frame_system::Pallet::<T>::inc_account_nonce(source);
		Ok(total_weight)
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

	fn compute_fee(len: u32, weight: Weight) -> BalanceOf<T> {
		let adjusted_weight_fee = T::WeightPrice::convert(weight);
		let length_fee = Self::length_to_fee(len);
		let base_fee =
			Self::weight_to_fee(T::BlockWeights::get().get(DispatchClass::Normal).base_extrinsic);
		let inclusion_fee = base_fee + length_fee + adjusted_weight_fee;
		inclusion_fee
	}

	/// Compute the length portion of a fee by invoking the configured `LengthToFee` impl.
	pub fn length_to_fee(length: u32) -> BalanceOf<T> {
		T::LengthToFee::weight_to_fee(&Weight::from_ref_time(length as u64))
	}

	/// Compute the unadjusted portion of the weight fee by invoking the configured `WeightToFee`
	/// impl. Note that the input `weight` is capped by the maximum block weight before computation.
	pub fn weight_to_fee(weight: Weight) -> BalanceOf<T> {
		// cap the weight to the maximum defined in runtime, otherwise it will be the
		// `Bounded` maximum of its data type, which is not desired.
		let capped_weight = weight.min(T::BlockWeights::get().max_block);
		T::WeightToFee::weight_to_fee(&capped_weight)
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
		from_address: &H160,
		to_address: &H160,
		amount: u128,
	) -> Result<Weight, RunnerError<Self::Error>> {
		let source = T::AddressMapping::into_account_id(*from_address);
		let target = T::AddressMapping::into_account_id(*to_address);
		let amount = amount.try_into().map_err(|_| RunnerError {
			error: Self::Error::InvalidType,
			weight: T::DbWeight::get().reads(2u64),
		})?;
		T::Currency::transfer(&source, &target, amount, ExistenceRequirement::AllowDeath).map_err(
			|_| RunnerError {
				error: Self::Error::BalanceLow,
				weight: T::DbWeight::get().reads(2u64),
			},
		)?;
		Ok(<T as pallet::Config>::WeightInfo::msg_send())
	}
}
