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

use signature::hazmat::PrehashVerifier;

/// Verify with secp256k1.
pub fn secp256k1_ecdsa_verify(pk: &[u8; 33], msg: &[u8], sig: &[u8]) -> bool {
	match (k256::ecdsa::VerifyingKey::from_sec1_bytes(pk), k256::ecdsa::Signature::from_slice(sig)) {
		(Ok(verifying_key), Ok(signature)) => verifying_key.verify_prehash(msg, &signature).is_ok(),
		_ => false,
	}
}
