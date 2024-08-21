// This file is part of Hrozion.

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

use cosmos_sdk_proto::{cosmos::bank::v1beta1::MsgSend, cosmwasm::wasm::v1::MsgStoreCode, Any};
use pallet_cosmos_types::msgservice::MsgHandler;
use pallet_cosmos_x_auth_signing::any_match;
use pallet_cosmos_x_bank::msgs::MsgSendHandler;
use pallet_cosmos_x_wasm::msgs::MsgStoreCodeHandler;
use sp_std::{boxed::Box, marker::PhantomData};

pub struct MsgServiceRouter<T>(PhantomData<T>);
impl<T> pallet_cosmos_types::msgservice::MsgServiceRouter for MsgServiceRouter<T>
where
	T: frame_system::Config + pallet_cosmos::Config,
{
	fn route(msg: &Any) -> Option<Box<dyn MsgHandler>> {
		any_match!(
			msg, {
				MsgSend => Some(Box::<MsgSendHandler<T>>::default()),
				MsgStoreCode => Some(Box::<MsgStoreCodeHandler<T>>::default()),
			},
			None
		)
	}
}
