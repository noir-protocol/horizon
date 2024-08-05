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

use cosmos_sdk_proto::Any;
use frame_support::weights::Weight;
use sp_runtime::RuntimeString;

pub struct MsgHandlerErrorInfo {
	pub weight: Weight,
	pub error: MsgHandlerError,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MsgHandlerError {
	InvalidMsg,
	ParseAmountError,
	Custom(RuntimeString),
}

pub trait MsgHandler {
	fn handle(&self, msg: &Any) -> Result<Weight, MsgHandlerErrorInfo>;
}

pub trait MsgServiceRouter {
	fn route(type_url: &str) -> Option<sp_std::boxed::Box<dyn MsgHandler>>;
}

impl MsgServiceRouter for () {
	fn route(_type_url: &str) -> Option<sp_std::boxed::Box<dyn MsgHandler>> {
		None
	}
}