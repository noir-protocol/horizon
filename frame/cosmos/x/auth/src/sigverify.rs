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

use alloc::string::ToString;
use core::marker::PhantomData;
use cosmos_sdk_proto::{
	cosmos::{
		crypto::{multisig::LegacyAminoPubKey, secp256k1},
		tx::v1beta1::{ModeInfo, SignerInfo, Tx},
	},
	prost::Message,
	Any,
};
use hp_io::cosmos::secp256k1_ecdsa_verify;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::{address::acc_address_from_bech32, handler::AnteDecorator};
use pallet_cosmos_x_auth_signing::{
	any_match,
	sign_mode_handler::{traits::SignModeHandler, SignerData},
	sign_verifiable_tx::traits::SigVerifiableTx,
};
use ripemd::Digest;
use sp_core::{sha2_256, Get, H160};
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};

pub struct SigVerificationDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for SigVerificationDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signatures = &tx.signatures;
		let signers = T::SigVerifiableTx::get_signers(tx)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

		let auth_info = tx
			.auth_info
			.as_ref()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
		if signatures.len() != signers.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		if signatures.len() != auth_info.signer_infos.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		for (i, sig) in signatures.iter().enumerate() {
			let signer = signers
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			let signer_info = auth_info
				.signer_infos
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			let (_hrp, signer_addr) = acc_address_from_bech32(signer)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			if signer_addr.len() != 20 {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
			}

			let signer_addr = H160::from_slice(&signer_addr);
			let (account, _) = pallet_cosmos::Pallet::<T>::account(&signer_addr);
			if signer_info.sequence > account.sequence {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Future));
			} else if signer_info.sequence < account.sequence {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale));
			}

			let public_key = signer_info
				.public_key
				.as_ref()
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
			let chain_id = T::ChainId::get().to_string();
			let signer_data = SignerData {
				address: signer.clone(),
				chain_id,
				account_number: 0,
				sequence: account.sequence,
				pub_key: public_key.clone(),
			};

			let sign_mode = signer_info
				.mode_info
				.as_ref()
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			Self::verify_signature(public_key, &signer_data, sign_mode, sig, tx)?;
		}

		Ok(ValidTransaction::default())
	}
}

impl<T> SigVerificationDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn verify_signature(
		public_key: &Any,
		signer_data: &SignerData,
		sign_mode: &ModeInfo,
		signature: &[u8],
		tx: &Tx,
	) -> Result<(), TransactionValidityError> {
		any_match!(
			public_key, {
				secp256k1::PubKey => {
					let public_key =
						secp256k1::PubKey::decode(&mut &*public_key.value).map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
						})?;
					let mut hasher = ripemd::Ripemd160::new();
					hasher.update(sha2_256(&public_key.key));
					let address = H160::from_slice(&hasher.finalize());

					let (_hrp, signer_addr) =
						acc_address_from_bech32(&signer_data.address).map_err(|_| {
							TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
						})?;

					if signer_addr.len() != 20 {
						return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
					}

					let signer_addr = H160::from_slice(&signer_addr);
					if signer_addr != address {
						return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
					}

					let sign_bytes = T::SignModeHandler::get_sign_bytes(sign_mode, signer_data, tx)
						.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Call))?;

					if !secp256k1_ecdsa_verify(signature, &sha2_256(&sign_bytes), &public_key.key) {
						return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
					}

					Ok(())
				}
			},
			Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))
		)
	}
}

pub struct ValidateSigCountDecorator<T>(core::marker::PhantomData<T>);

impl<T> AnteDecorator for ValidateSigCountDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let mut sig_count = 0u64;

		let auth_info = tx
			.auth_info
			.as_ref()
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
		for SignerInfo { public_key, .. } in auth_info.signer_infos.iter() {
			let public_key = public_key
				.as_ref()
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
			sig_count = sig_count.saturating_add(Self::count_sub_keys(public_key)?);

			if sig_count > T::TxSigLimit::get() {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
			}
		}

		Ok(ValidTransaction::default())
	}
}

impl<T> ValidateSigCountDecorator<T> {
	fn count_sub_keys(pubkey: &Any) -> Result<u64, TransactionValidityError> {
		// TODO: Support legacy multi signatures.
		if LegacyAminoPubKey::decode(&mut &*pubkey.value).is_ok() {
			Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof))
		} else {
			Ok(1)
		}
	}
}

pub struct IncrementSequenceDecorator<T>(core::marker::PhantomData<T>);

impl<T> AnteDecorator for IncrementSequenceDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signers = T::SigVerifiableTx::get_signers(tx)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
		for signer in signers.iter() {
			let account = T::AddressMapping::from_bech32(signer)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
			frame_system::pallet::Pallet::<T>::inc_account_nonce(account);
		}

		Ok(ValidTransaction::default())
	}
}
