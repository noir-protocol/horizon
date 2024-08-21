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

use cosmos_sdk_proto::{cosmwasm::wasm::v1::MsgStoreCode, prost::Message, Any};
use frame_support::weights::Weight;
use pallet_cosmos_types::{
	errors::RootError, events::CosmosEvent, msgservice::MsgHandlerErrorInfo,
};
use sp_std::{marker::PhantomData, vec::Vec};

pub struct MsgStoreCodeHandler<T>(PhantomData<T>);

impl<T> Default for MsgStoreCodeHandler<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<T> pallet_cosmos_types::msgservice::MsgHandler for MsgStoreCodeHandler<T> {
	fn handle(&self, msg: &Any) -> Result<(Weight, Vec<CosmosEvent>), MsgHandlerErrorInfo> {
		let total_weight = Weight::zero();

		let MsgStoreCode { sender: _, wasm_byte_code: _, instantiate_permission: _ } =
			MsgStoreCode::decode(&mut &*msg.value).map_err(|_| MsgHandlerErrorInfo {
				weight: total_weight,
				error: RootError::TxDecodeError.into(),
			})?;

		// TODO: Implements store wasm code with pallet_cosmwasm
		Err(MsgHandlerErrorInfo { weight: total_weight, error: RootError::UnknownRequest.into() })
	}
}
