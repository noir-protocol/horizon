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
use pallet_cosmos_types::{address::acc_address_from_bech32, any_match, handler::AnteDecorator};
use pallet_cosmos_x_auth_signing::{
	sign_mode_handler::{traits::SignModeHandler, SignerData},
	sign_verifiable_tx::traits::SigVerifiableTx,
};
use ripemd::Digest;
use sp_core::{sha2_256, Get, H160};
use sp_runtime::{
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
	SaturatedConversion,
};

pub struct SigVerificationDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for SigVerificationDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signatures = &tx.signatures;
		let signers =
			T::SigVerifiableTx::get_signers(tx).map_err(|_| InvalidTransaction::BadSigner)?;

		let auth_info = tx.auth_info.as_ref().ok_or(InvalidTransaction::BadSigner)?;
		if signatures.len() != signers.len() {
			return Err(InvalidTransaction::BadSigner.into());
		}

		if signatures.len() != auth_info.signer_infos.len() {
			return Err(InvalidTransaction::BadSigner.into());
		}

		for (i, sig) in signatures.iter().enumerate() {
			let signer = signers.get(i).ok_or(InvalidTransaction::BadSigner)?;

			let signer_info = auth_info.signer_infos.get(i).ok_or(InvalidTransaction::BadSigner)?;

			let (_hrp, signer_addr_raw) =
				acc_address_from_bech32(signer).map_err(|_| InvalidTransaction::BadSigner)?;

			if signer_addr_raw.len() != 20 {
				return Err(InvalidTransaction::BadSigner.into());
			}

			let who = T::AddressMapping::into_account_id(H160::from_slice(&signer_addr_raw));
			let sequence = frame_system::Pallet::<T>::account_nonce(&who).saturated_into();

			if signer_info.sequence > sequence {
				return Err(InvalidTransaction::Future.into());
			} else if signer_info.sequence < sequence {
				return Err(InvalidTransaction::Stale.into());
			}

			let public_key =
				signer_info.public_key.as_ref().ok_or(InvalidTransaction::BadSigner)?;
			let chain_id = T::ChainId::get().to_string();
			let signer_data = SignerData {
				address: signer.clone(),
				chain_id,
				account_number: 0,
				sequence: signer_info.sequence,
				pub_key: public_key.clone(),
			};

			let sign_mode = signer_info.mode_info.as_ref().ok_or(InvalidTransaction::BadSigner)?;

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
							InvalidTransaction::BadSigner
						})?;
					let mut hasher = ripemd::Ripemd160::new();
					hasher.update(sha2_256(&public_key.key));
					let address = H160::from_slice(&hasher.finalize());

					let (_hrp, signer_addr_raw) =
						acc_address_from_bech32(&signer_data.address).map_err(|_| {
							InvalidTransaction::BadSigner
						})?;

					if signer_addr_raw.len() != 20 {
						return Err(InvalidTransaction::BadSigner.into());
					}

					if  H160::from_slice(&signer_addr_raw) != address {
						return Err(InvalidTransaction::BadSigner.into());
					}

					let sign_bytes = T::SignModeHandler::get_sign_bytes(sign_mode, signer_data, tx)
						.map_err(|_| InvalidTransaction::Call)?;

					if !secp256k1_ecdsa_verify(signature, &sha2_256(&sign_bytes), &public_key.key) {
						return Err(InvalidTransaction::BadProof.into());
					}

					Ok(())
				}
			},
			Err(InvalidTransaction::BadSigner.into())
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

		let auth_info = tx.auth_info.as_ref().ok_or(InvalidTransaction::BadSigner)?;
		for SignerInfo { public_key, .. } in auth_info.signer_infos.iter() {
			let public_key = public_key.as_ref().ok_or(InvalidTransaction::BadSigner)?;
			sig_count = sig_count.saturating_add(Self::count_sub_keys(public_key)?);

			if sig_count > T::TxSigLimit::get() {
				return Err(InvalidTransaction::BadProof.into());
			}
		}

		Ok(ValidTransaction::default())
	}
}

impl<T> ValidateSigCountDecorator<T> {
	fn count_sub_keys(pubkey: &Any) -> Result<u64, TransactionValidityError> {
		// TODO: Support legacy multi signatures.
		if LegacyAminoPubKey::decode(&mut &*pubkey.value).is_ok() {
			Err(InvalidTransaction::BadProof.into())
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
		let signers = T::SigVerifiableTx::get_signers(tx).map_err(|_| InvalidTransaction::Call)?;
		for signer in signers.iter() {
			let (_hrp, address_raw) =
				acc_address_from_bech32(signer).map_err(|_| InvalidTransaction::BadSigner)?;
			if address_raw.len() != 20 {
				return Err(InvalidTransaction::BadSigner.into());
			}

			let account = T::AddressMapping::into_account_id(H160::from_slice(&address_raw));
			frame_system::pallet::Pallet::<T>::inc_account_nonce(account);
		}

		Ok(ValidTransaction::default())
	}
}
