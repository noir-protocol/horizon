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
	let msg = match Message::from_digest_slice(msg) {
		Ok(v) => v,
		Err(_) => return false,
	};
	let pub_key = match PublicKey::from_slice(pub_key) {
		Ok(v) => v,
		Err(_) => return false,
	};

	sig.verify(&msg, &pub_key).is_ok()
}

#[cfg(test)]
mod tests {
	use super::secp256k1_ecdsa_verify;

	#[test]
	fn secp256k1_ecdsa_verify_test() {
		let pub_key =
			hex::decode("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")
				.unwrap();
		let sig = hex::decode("605c888fa01a5e706ed0a4a0c93daced16882abe6c630c71793917d606b3a84b37f4f80026721d425eb1b83a688a2f4cbcbfd213c7443359851ceee79771d226").unwrap();
		let msg = hex::decode("714d4bdfdbd0bd630ebdf93b1f6eba7d3c752e92bbab6c9d3d9c93e1777348bb")
			.unwrap();

		assert!(secp256k1_ecdsa_verify(&sig, &msg, &pub_key));
	}
}
