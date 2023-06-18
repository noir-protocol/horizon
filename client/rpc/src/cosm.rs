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

pub use hp_rpc::CosmosRuntimeRPCApi;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use primitive_types::U256;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

#[rpc(server)]
#[async_trait]
pub trait CosmApi {
	/// Returns account.
	#[method(name = "cosm_broadcast_tx")]
	async fn broadcast_tx(&self, tx_bytes: Vec<u8>) -> RpcResult<U256>;
}

pub struct Cosm<B: BlockT, C> {
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, C> Cosm<B, C> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

#[async_trait]
impl<B, C> CosmApiServer for Cosm<B, C>
where
	B: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<B>,
	C: HeaderBackend<B> + 'static,
	C::Api: CosmosRuntimeRPCApi<B>,
{
	async fn broadcast_tx(&self, tx_bytes: Vec<u8>) -> RpcResult<U256> {
		let tx_hash: [u8; 32] = [0u8; 32]; 
		Ok(tx_hash.into())
	}
}
