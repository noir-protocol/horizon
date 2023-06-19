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
	types::{error::CallError, ErrorObject},
};
use primitive_types::H256;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{app_crypto::sp_core::hashing::sha2_256, traits::Block as BlockT};
use std::{marker::PhantomData, sync::Arc};

#[rpc(server)]
#[async_trait]
pub trait CosmApi {
	/// Returns account.
	#[method(name = "cosm_broadcast_tx")]
	async fn broadcast_tx(&self, tx_bytes: Vec<u8>) -> RpcResult<H256>;
}

pub struct Cosm<P, B: BlockT, C> {
	pool: Arc<P>,
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<P, B: BlockT, C> Cosm<P, B, C> {
	pub fn new(pool: Arc<P>, client: Arc<C>) -> Self {
		Self { pool, client, _marker: Default::default() }
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
impl<P, B, C> CosmApiServer for Cosm<P, B, C>
where
	B: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<B>,
	C: HeaderBackend<B> + 'static,
	C::Api: CosmosRuntimeRPCApi<B>,
	P: TransactionPool<Block = B> + 'static,
{
	async fn broadcast_tx(&self, tx_bytes: Vec<u8>) -> RpcResult<H256> {
		let tx = cosmrs::Tx::from_bytes(&tx_bytes[..]).map_err(|e| {
			CallError::Custom(ErrorObject::owned(
				Error::RuntimeError.into(),
				"Unable to decode tx.",
				Some(e.to_string()),
			))
		})?;
		println!("{:?}", tx);
		let tx_hash = sha2_256(&tx_bytes[..]);
		Ok(tx_hash.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use base64::{engine::general_purpose, Engine};
	use cosmrs::tx::MessageExt;

	#[test]
	fn test_decode_tx() {
		let tx_bytes =
			"CpcBCpIBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnIKLWNvc21vczFodzRkNGRzeHgyc2xoMzlxdzVoY3JxbjUya2FzNW5sNTYzbmVkcBItY29zbW9zMXhtZ2d4Y3duZ2Nta3JscmM2cnhrZHU2ZGpzczc4dTV2ZTR2dmc5GhIKBGNjZHQSCjEwMDAwMDAwMDASABJlClEKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiED0Ttls626xrRUN1BnwRYPKGsFMtIWWUFEojTiWQf6j80SBAoCCH8YngMSEAoKCgRjY2R0EgIyNRCgjQYaQMuNWIlLtLg2pEUPDeqNv7vmvBU0HWGPndjjj1/fHGY5BGUjvnWN1wgRGcl92adj3k3WxuZqXN2PEmM4krsedPE=";
		let tx_bytes = general_purpose::STANDARD.decode(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes[..]).unwrap();
		let message =
			cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(&tx.body.messages[0]).unwrap();
		assert_eq!(message.from_address, "cosmos1hw4d4dsxx2slh39qw5hcrqn52kas5nl563nedp");
		assert_eq!(message.to_address, "cosmos1xmggxcwngcmkrlrc6rxkdu6djss78u5ve4vvg9");
		assert_eq!(message.amount[0].denom, "ccdt");
		assert_eq!(message.amount[0].amount, "1000000000");
	}
}
