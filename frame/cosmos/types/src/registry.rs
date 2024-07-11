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

use crate::{
	error::DecodeError,
	legacy::LegacyMsg,
	tx::{AccountId, Any},
};
use std::sync::OnceLock;

pub static REGISTRY: OnceLock<Box<dyn Registry + Send + Sync + 'static>> = OnceLock::new();

pub trait Registry {
	fn signers(&self, any: &Any) -> Result<Vec<AccountId>, DecodeError>;
	fn transcode(&self, any: &Any) -> Result<Vec<u8>, DecodeError>;
	fn legacy_msg(&self, any: &Any) -> Result<LegacyMsg, DecodeError>;
}

#[macro_export]
macro_rules! register_cosmos_types {
	($($t:ty),* $(,)?) => {
		pub(crate) mod cosmos_type_registry {
			use $crate::{error::DecodeError, msgs::Msg, tx::{AccountId, Any}};
			use parity_scale_codec::Encode;

			pub struct Registry;

			pub fn init() {
				let _ = $crate::registry::REGISTRY.set(Box::new(Registry));
			}

			impl $crate::registry::Registry for Registry {
				fn signers(&self, any: &Any) -> Result<Vec<AccountId>, DecodeError> {
					match &any.type_url[..] {
						$(<$t as Msg>::TYPE_URL => Ok(<$t>::try_from(any.clone())?.get_signers()),)*
						_ => Err(DecodeError::InvalidTypeUrl),
					}
				}

				fn transcode(&self, any: &Any) -> Result<Vec<u8>, DecodeError> {
					match &any.type_url[..] {
						$(<$t as Msg>::TYPE_URL => Ok(<$t>::try_from(any.clone())?.encode()),)*
						_ => Err(DecodeError::InvalidTypeUrl),
					}
				}

				fn legacy_msg(&self, any: &Any) -> Result<pallet_cosmos_types::legacy::LegacyMsg, DecodeError> {
					match &any.type_url[..] {
						$(<$t as Msg>::TYPE_URL => Ok(<$t>::legacy_msg(any.clone())?),)*
						_ => Err(DecodeError::InvalidTypeUrl),
					}
				}
			}
		}
	};
}

pub use register_cosmos_types;
