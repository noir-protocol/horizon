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

use fc_rpc::internal_err;
use futures::future::TryFutureExt;
use hp_cosmos;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::{Bytes, H256};
use sp_runtime::{
	generic::BlockId, traits::Block as BlockT, transaction_validity::TransactionSource,
};
use std::{marker::PhantomData, sync::Arc};

#[rpc(server)]
#[async_trait]
pub trait CosmApi {
	#[method(name = "cosm_broadcast_tx")]
	async fn broadcast_tx(&self, tx_bytes: Bytes) -> RpcResult<H256>;
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

#[async_trait]
impl<P, B, C> CosmApiServer for Cosm<P, B, C>
where
	B: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<B>,
	C: HeaderBackend<B> + 'static,
	C::Api: hp_rpc::ConvertTxRuntimeApi<B>,
	P: TransactionPool<Block = B> + 'static,
{
	async fn broadcast_tx(&self, tx_bytes: Bytes) -> RpcResult<H256> {
		use hp_rpc::ConvertTxRuntimeApi;

		let tx = cosmrs::Tx::from_bytes(&tx_bytes)
			.map_err(|e| internal_err(format!("Invalid transaction. error={}", e)))?;
		let tx: hp_cosmos::Tx = tx
			.try_into()
			.map_err(|e| internal_err(format!("Invalid transaction. error={}", e)))?;
		let block_hash = self.client.info().best_hash;
		let extrinsic = self
			.client
			.runtime_api()
			.convert_tx(block_hash, tx.clone())
			.map_err(|_| internal_err("Cannot access runtime api."))?;
		self.pool
			.submit_one(&BlockId::Hash(block_hash), TransactionSource::Local, extrinsic)
			.map_ok(move |_| tx.hash.into())
			.map_err(|e| internal_err(e.to_string()))
			.await
	}
}

#[cfg(test)]
mod tests {
	use base64::{engine::general_purpose, Engine};
	use cosmrs::tx::MessageExt;

	#[test]
	fn test_decode_tx() {
		let tx_bytes =
			"CpcBCpIBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnIKLWNvc21vczFodzRkNGRzeHgyc2xoMzlxdzVoY3JxbjUya2FzNW5sNTYzbmVkcBItY29zbW9zMXhtZ2d4Y3duZ2Nta3JscmM2cnhrZHU2ZGpzczc4dTV2ZTR2dmc5GhIKBGNjZHQSCjEwMDAwMDAwMDASABJlClEKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiED0Ttls626xrRUN1BnwRYPKGsFMtIWWUFEojTiWQf6j80SBAoCCH8YngMSEAoKCgRjY2R0EgIyNRCgjQYaQMuNWIlLtLg2pEUPDeqNv7vmvBU0HWGPndjjj1/fHGY5BGUjvnWN1wgRGcl92adj3k3WxuZqXN2PEmM4krsedPE=";
		let tx_bytes = general_purpose::STANDARD.decode(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes).unwrap();
		let message =
			cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(&tx.body.messages[0]).unwrap();
		assert_eq!(message.from_address, "cosmos1hw4d4dsxx2slh39qw5hcrqn52kas5nl563nedp");
		assert_eq!(message.to_address, "cosmos1xmggxcwngcmkrlrc6rxkdu6djss78u5ve4vvg9");
		assert_eq!(message.amount[0].denom, "ccdt");
		assert_eq!(message.amount[0].amount, "1000000000");
	}
}
