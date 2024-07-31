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

use cosmos_sdk_proto::cosmos::tx::v1beta1::Tx;
use hp_io::cosmos::secp256k1_ecdsa_verify;
use pallet_cosmos::AddressMapping;
use pallet_cosmos_types::handler::AnteDecorator;
use sp_core::{sha2_256, Get, H160};
use sp_runtime::transaction_validity::{
	InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};
use sp_std::marker::PhantomData;

pub struct SigVerificationDecorator<T>(PhantomData<T>);

impl<T> AnteDecorator for SigVerificationDecorator<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let signatures = &tx.signatures;

		let signers = hp_io::cosmos::get_signers(tx)
			.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

		if signatures.len() != signers.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		let signer_infos = &tx.auth_info.signer_infos;
		if signatures.len() != signer_infos.len() {
			return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
		}

		for (i, sig) in signatures.iter().enumerate() {
			let signer = signers
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			// TODO: Support other types of Signers as well
			let signer_info = signer_infos
				.get(i)
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadSigner))?;

			if let Some(SignerPublicKey::Single(PublicKey::Secp256k1(public_key))) =
				signer_info.public_key
			{
				let address: H160 = hp_io::cosmos::ripemd160(&sha2_256(&public_key)).into();
				if signer.address != address {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
				}

				let (account, _) = pallet_cosmos::Pallet::<T>::account(&signer.address);
				if signer_info.sequence > account.sequence {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::Future));
				} else if signer_info.sequence < account.sequence {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::Stale));
				}

				let chain_id = T::ChainId::get();
				let bytes = match &signer_info.mode_info {
					pallet_cosmos_types::tx::ModeInfo::Single(single) => match single.mode {
						pallet_cosmos_types::tx::SignMode::Direct =>
							hp_io::cosmos::sign_bytes(&tx.raw, &chain_id, 0u64),
						pallet_cosmos_types::tx::SignMode::LegacyAminoJson =>
							hp_io::cosmos::std_sign_bytes(
								&tx.raw,
								&chain_id,
								0u64,
								signer_info.sequence,
							),
						_ => None,
					},
				}
				.ok_or(TransactionValidityError::Invalid(InvalidTransaction::BadProof))?;

				let hash = sp_core::sha2_256(&bytes);
				if !secp256k1_ecdsa_verify(sig, &hash, &public_key) {
					return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
				}
			} else {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::BadSigner));
			}
		}

		Ok(ValidTransaction::default())
	}
}

pub struct ValidateSigCountDecorator<T>(sp_std::marker::PhantomData<T>);

impl<T> AnteDecorator for ValidateSigCountDecorator<T>
where
	T: pallet_cosmos::Config,
{
	fn ante_handle(tx: &Tx, _simulate: bool) -> TransactionValidity {
		let mut sig_count = 0u64;
		for SignerInfo { public_key, .. } in &tx.auth_info.signer_infos {
			sig_count = sig_count.saturating_add(Self::count_sub_keys(public_key.clone()));

			if sig_count > T::TxSigLimit::get() {
				return Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof));
			}
		}

		Ok(ValidTransaction::default())
	}
}

impl<T> ValidateSigCountDecorator<T> {
	fn count_sub_keys(public_key: Option<SignerPublicKey>) -> u64 {
		// TODO: Support legacy multi signatures.
		match public_key {
			Some(SignerPublicKey::Single(_)) => 1,
			None => 0,
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
