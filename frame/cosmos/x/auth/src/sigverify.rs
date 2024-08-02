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

use bech32::FromBase32;
use cosmos_sdk_proto::{
	cosmos::{
		crypto::{multisig::LegacyAminoPubKey, secp256k1},
		tx::{
			signing::v1beta1::SignMode,
			v1beta1::{
				mode_info::{self, Sum},
				ModeInfo, SignDoc, SignerInfo, Tx,
			},
		},
	},
	prost::{alloc::string::String, Message},
	Any,
};
use hp_io::cosmos::secp256k1_ecdsa_verify;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::handler::AnteDecorator;
use pallet_cosmos_x_auth_signing::{
	sign_mode_handler::{SignModeHander, SignerData},
	sign_verifiable_tx::SigVerifiableTx,
};
use sp_core::{sha2_256, Get, H160};
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};
use sp_std::{marker::PhantomData, vec::Vec};

pub const SECP256K1_TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";
pub struct SigVerificationDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for SigVerificationDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signatures = &tx.signatures;
		let signers = T::SigVerifiableTx::get_signers(tx);

		let auth_info = tx
			.auth_info
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
		if signatures.len() != signers.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		let signer_infos = auth_info.signer_infos;

		if signatures.len() != signer_infos.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		for (i, sig) in signatures.iter().enumerate() {
			let signer = signers
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			let signer_info = signer_infos
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			let (_, signer_addr, _) = bech32::decode(signer)
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
			let signer_addr = Vec::<u8>::from_base32(&signer_addr).unwrap();
			let signer_addr = H160::from_slice(&signer_addr);

			let (account, _) = pallet_cosmos::Pallet::<T>::account(&signer_addr);
			if signer_info.sequence > account.sequence {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Future));
			} else if signer_info.sequence < account.sequence {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale));
			}

			let public_key = signer_info
				.public_key
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
			let chain_id = T::ChainId::get();
			let chain_id = String::from_utf8(chain_id.to_vec()).unwrap();
			let signer_data = SignerData {
				address: signer.clone(),
				chain_id,
				account_number: 0,
				sequence: account.sequence,
				pub_key: public_key,
			};

			let sign_mode = signer_info
				.mode_info
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			Self::verify_signature(&public_key, &signer_data, &sign_mode, sig, tx)?;
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
		match public_key.type_url.as_str() {
			SECP256K1_TYPE_URL => {
				let public_key =
					secp256k1::PubKey::decode(&mut &*public_key.value).map_err(|_| {
						TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
					})?;
				let address: H160 = hp_io::cosmos::ripemd160(&sha2_256(&public_key.key)).into();

				let (_, signer_addr, _) = bech32::decode(&signer_data.address).map_err(|_| {
					TransactionValidityError::Invalid(InvalidTransaction::BadSigner)
				})?;
				let signer_addr = Vec::<u8>::from_base32(&signer_addr).unwrap();
				let signer_addr = H160::from_slice(&signer_addr);

				if signer_addr != address {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
				}

				let sign_bytes = T::SignModeHander::sign_bytes(sign_mode, signer_data, tx);
				let msg = sha2_256(&sign_bytes);

				if !secp256k1_ecdsa_verify(signature, &msg, &public_key.key) {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
				}

				Ok(())
			},
			_ => return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner)),
		}
	}
}

pub struct ValidateSigCountDecorator<T>(sp_std::marker::PhantomData<T>);

impl<T> AnteDecorator for ValidateSigCountDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let mut sig_count = 0u64;
		if let Some(auth_info) = &tx.auth_info {
			for SignerInfo { public_key, .. } in &auth_info.signer_infos {
				if let Some(public_key) = public_key {
					sig_count = sig_count.saturating_add(Self::count_sub_keys(&public_key)?);
				}

				if sig_count > T::TxSigLimit::get() {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
				}
			}
		} else {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
		}

		Ok(ValidTransaction::default())
	}
}

impl<T> ValidateSigCountDecorator<T> {
	fn count_sub_keys(pubkey: &Any) -> Result<u64, TransactionValidityError> {
		// TODO: Support legacy multi signatures.
		if let Ok(pubkey) = LegacyAminoPubKey::decode(&mut &*pubkey.value) {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
		} else {
			Ok(1)
		}
	}
}

pub struct IncrementSequenceDecorator<T>(sp_std::marker::PhantomData<T>);
impl<T> AnteDecorator for IncrementSequenceDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signers = hp_io::cosmos::get_signers(tx)
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;
		for signer in signers {
			let account = T::AddressMapping::into_account_id(signer.address);
			frame_system::pallet::Pallet::<T>::inc_account_nonce(account);
		}

		Ok(ValidTransaction::default())
	}
}
