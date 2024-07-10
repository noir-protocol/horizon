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

use crate::{error::DecodeError, msgs::Msg, tx::Any};
use parity_scale_codec::Encode;
use std::sync::OnceLock;

pub static TRANSCODER: OnceLock<Box<Transcoder>> = OnceLock::new();

pub type Transcoder = dyn Fn(&[u8], &[u8]) -> Result<Vec<u8>, DecodeError> + Send + Sync + 'static;

pub trait Transcode {
	fn transcode(type_url: &[u8], value: &[u8]) -> Result<Vec<u8>, DecodeError>;
}

impl<T: TryFrom<Any, Error = DecodeError> + Msg + Encode> Transcode for T {
	fn transcode(type_url: &[u8], value: &[u8]) -> Result<Vec<u8>, DecodeError> {
		let any = Any { type_url: type_url.to_vec(), value: value.to_vec() };
		let msg = T::try_from(any)?;
		Ok(msg.encode())
	}
}

#[macro_export]
macro_rules! register_protobuf_types {
	($($t:ty),* $(,)?) => {
		let _ = $crate::protobuf::TRANSCODER.set(Box::new(|type_url, value| {
			match type_url {
				$(<$t as $crate::msgs::Msg>::TYPE_URL =>
					<$t as $crate::protobuf::Transcode>::transcode(type_url, value),)*
				_ => Err(pallet_cosmos_types::error::DecodeError::InvalidTypeUrl),
			}
		}));
	};
}
