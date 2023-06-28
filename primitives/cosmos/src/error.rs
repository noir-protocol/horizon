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

use core::fmt::Display;

#[derive(Debug)]
pub enum DecodeTxError {
	InvalidTxData,
	UnsupportedSigner,
	InvalidMsgData,
	UnsupportedMsgType,
	InvalidSignDoc,
}

impl Display for DecodeTxError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			DecodeTxError::InvalidTxData => write!(f, "{}", "INVALID_TX_DATA"),
			DecodeTxError::UnsupportedSigner => write!(f, "{}", "UNSUPPORTED_SIGNER"),
			DecodeTxError::InvalidMsgData => write!(f, "{}", "INVALID_MSG_DATA"),
			DecodeTxError::UnsupportedMsgType => write!(f, "{}", "UNSUPPORTED_MSG_TYPE"),
			DecodeTxError::InvalidSignDoc => write!(f, "{}", "INVALID_SIGN_DOC"),
		}
	}
}
