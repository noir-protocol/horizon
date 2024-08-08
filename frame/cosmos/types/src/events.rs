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

#[cfg(feature = "with-codec")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "with-codec")]
use scale_info::TypeInfo;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

pub const EVENT_TYPE_MESSAGE: &str = "message";

pub const ATTRIBUTE_KEY_SENDER: &str = "sender";
pub const ATTRIBUTE_KEY_AMOUNT: &str = "amount";

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct AbciEvent {
	#[serde(rename = "type")]
	pub r#type: Vec<u8>,
	pub attributes: Vec<EventAttribute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(Encode, Decode, TypeInfo))]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct EventAttribute {
	pub key: Vec<u8>,
	pub value: Vec<u8>,
}
