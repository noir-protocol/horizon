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

#[cfg(feature = "std")]
use secp256k1::{ecdsa::Signature, Message, PublicKey};

/// Verify a secp256k1 ECDSA signature.
#[cfg(feature = "std")]
pub fn secp256k1_ecdsa_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool {
	let sig = match Signature::from_compact(sig) {
		Ok(v) => v,
		Err(_) => return false,
	};
	let msg = match Message::from_slice(msg) {
		Ok(v) => v,
		Err(_) => return false,
	};
	let pub_key = match PublicKey::from_slice(pub_key) {
		Ok(v) => v,
		Err(_) => return false,
	};

	sig.verify(&msg, &pub_key).is_ok()
}
