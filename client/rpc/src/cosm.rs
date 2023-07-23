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

use fc_rpc::internal_err;
use futures::future::TryFutureExt;
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
	#[method(name = "cosm_broadcastTx")]
	async fn broadcast_tx(&self, tx_bytes: Bytes) -> RpcResult<H256>;
}

pub struct Cosm<B: BlockT, C, P> {
	chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	pool: Arc<P>,
	client: Arc<C>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, C, P> Cosm<B, C, P> {
	pub fn new(
		chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
		pool: Arc<P>,
		client: Arc<C>,
	) -> Self {
		Self { chain_spec, pool, client, _marker: Default::default() }
	}
}

#[async_trait]
impl<B, C, P> CosmApiServer for Cosm<B, C, P>
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

		let tx =
			cosmrs::Tx::from_bytes(&tx_bytes).map_err(|_| internal_err("Invalid transaction."))?;
		let chain_id = self.chain_spec.id();
		let tx_len = tx_bytes
			.len()
			.try_into()
			.map_err(|_| internal_err("Transaction is too long."))?;
		let tx = hp_cosmos::Tx::new(tx, chain_id, tx_len)
			.map_err(|_| internal_err("Invalid transaction."))?;
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
	use base64ct::{Base64, Encoding};
	use cosmrs::tx::MessageExt;

	#[test]
	fn test_decode_tx() {
		let tx_bytes =
			"CpcBCpIBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEnIKLWNvc21vczFodzRkNGRzeHgyc2xoMzlxdzVoY3JxbjUya2FzNW5sNTYzbmVkcBItY29zbW9zMXhtZ2d4Y3duZ2Nta3JscmM2cnhrZHU2ZGpzczc4dTV2ZTR2dmc5GhIKBGNjZHQSCjEwMDAwMDAwMDASABJlClEKRgofL2Nvc21vcy5jcnlwdG8uc2VjcDI1NmsxLlB1YktleRIjCiED0Ttls626xrRUN1BnwRYPKGsFMtIWWUFEojTiWQf6j80SBAoCCH8YngMSEAoKCgRjY2R0EgIyNRCgjQYaQMuNWIlLtLg2pEUPDeqNv7vmvBU0HWGPndjjj1/fHGY5BGUjvnWN1wgRGcl92adj3k3WxuZqXN2PEmM4krsedPE=";
		let tx_bytes = Base64::decode_vec(tx_bytes).unwrap();
		let tx = cosmrs::Tx::from_bytes(&tx_bytes).unwrap();
		let message =
			cosmrs::proto::cosmos::bank::v1beta1::MsgSend::from_any(&tx.body.messages[0]).unwrap();
		assert_eq!(message.from_address, "cosmos1hw4d4dsxx2slh39qw5hcrqn52kas5nl563nedp");
		assert_eq!(message.to_address, "cosmos1xmggxcwngcmkrlrc6rxkdu6djss78u5ve4vvg9");
		assert_eq!(message.amount[0].denom, "ccdt");
		assert_eq!(message.amount[0].amount, "1000000000");
	}
}
