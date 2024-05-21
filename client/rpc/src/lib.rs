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

#![deny(unused_crate_dependencies)]

mod cosm;

pub use cosm::{Cosm, CosmApiServer};

pub fn err<T: ToString>(
	code: i32,
	message: T,
	data: Option<&[u8]>,
) -> jsonrpsee::types::ErrorObjectOwned {
	jsonrpsee::types::error::ErrorObject::owned(
		code,
		message.to_string(),
		data.map(|bytes| {
			jsonrpsee::core::to_json_raw_value(&format!("0x{}", hex::encode(bytes)))
				.expect("fail to serialize data")
		}),
	)
}

pub fn request_err<T: ToString>(message: T) -> jsonrpsee::types::ErrorObjectOwned {
	err(jsonrpsee::types::error::INVALID_REQUEST_CODE, message, None)
}

pub fn internal_err<T: ToString>(message: T) -> jsonrpsee::types::ErrorObjectOwned {
	err(jsonrpsee::types::error::INTERNAL_ERROR_CODE, message, None)
}
