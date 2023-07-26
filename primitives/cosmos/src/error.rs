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
	EmptyFeeAmount,
	EmptyMessages,
	EmptyMsgSendAmount,
	EmptySignatures,
	EmptySigners,
	InvalidMsgData,
	InvalidSignDoc,
	InvalidTxData,
	TooManyFeeAmount,
	TooManyMsgSendAmount,
	TooManyMessages,
	TooManySignatures,
	TooManySigners,
	UnsupportedMsgType,
	UnsupportedSignerType,
	UnsupportedSignMode,
}

impl Display for DecodeTxError {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			DecodeTxError::EmptyFeeAmount => write!(f, "{}", "EMPTY_FEE_AMOUNT"),
			DecodeTxError::EmptyMessages => write!(f, "{}", "EMPTY_MESSAGES"),
			DecodeTxError::EmptyMsgSendAmount => write!(f, "{}", "EMPTY_MSG_SEND_AMOUNT"),
			DecodeTxError::EmptySignatures => write!(f, "{}", "EMPTY_SIGNATURES"),
			DecodeTxError::EmptySigners => write!(f, "{}", "EMPTY_SIGNERS"),
			DecodeTxError::InvalidMsgData => write!(f, "{}", "INVALID_MSG_DATA"),
			DecodeTxError::InvalidSignDoc => write!(f, "{}", "INVALID_SIGN_DOC"),
			DecodeTxError::InvalidTxData => write!(f, "{}", "INVALID_TX_DATA"),
			DecodeTxError::TooManyFeeAmount => write!(f, "{}", "TOO_MANY_FEE_AMOUNT"),
			DecodeTxError::TooManyMessages => write!(f, "{}", "TOO_MANY_MESSAGES"),
			DecodeTxError::TooManyMsgSendAmount => write!(f, "{}", "TOO_MANY_MSG_SEND_AMOUNT"),
			DecodeTxError::TooManySignatures => write!(f, "{}", "TOO_MANY_SIGNATURES"),
			DecodeTxError::TooManySigners => write!(f, "{}", "TOO_MANY_SIGNERS"),
			DecodeTxError::UnsupportedMsgType => write!(f, "{}", "UNSUPPORTED_MSG_TYPE"),
			DecodeTxError::UnsupportedSignerType => write!(f, "{}", "UNSUPPORTED_SIGNER_TYPE"),
			DecodeTxError::UnsupportedSignMode => write!(f, "{}", "UNSUPPORTED_SIGN_MODE"),
		}
	}
}

#[repr(u8)]
#[derive(num_enum::FromPrimitive, num_enum::IntoPrimitive)]
pub enum TransactionValidationError {
	#[allow(dead_code)]
	#[num_enum(default)]
	InvalidSignature = 2,
	UnsupportedSignerType = 128,
}
