extern crate alloc;

use crate::cosmwasm_api::CosmwasmApiServer;
use alloc::sync::Arc;
use codec::Codec;
use core::{cmp::Ord, fmt::Display, marker::PhantomData, str::FromStr};
use cosmwasm_runtime_api::CosmwasmRuntimeApi;
use jsonrpsee::{
	core::RpcResult,
	proc_macros::rpc,
	types::{ErrorObject, ErrorObjectOwned},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[allow(clippy::too_many_arguments)]
mod cosmwasm_api {
	use super::*;
	#[rpc(client, server)]
	pub trait CosmwasmApi<BlockHash, AccountId, AssetId, Balance, Error>
	where
		AccountId: FromStr + Display,
		AssetId: FromStr + Display + Ord,
		Balance: FromStr + Display,
	{
		#[method(name = "cosmwasm_query")]
		fn query(
			&self,
			contract: AccountId,
			gas: u64,
			query_request: Vec<u8>,
			at: Option<BlockHash>,
		) -> RpcResult<Vec<u8>>;
	}
}

pub struct Cosmwasm<C, Block> {
	client: Arc<C>,
	_marker: PhantomData<Block>,
}

impl<C, M> Cosmwasm<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

fn runtime_error_into_rpc_error<E: ToString>(e: E) -> ErrorObjectOwned {
	ErrorObject::owned(
		9876, // no real reason for this value
		e.to_string(),
		None::<()>,
	)
}

impl<C, Block, AccountId, AssetId, Balance, Error>
	CosmwasmApiServer<<Block as BlockT>::Hash, AccountId, AssetId, Balance, Error>
	for Cosmwasm<C, (Block, AccountId, AssetId, Balance, Error)>
where
	Block: BlockT,
	AccountId: Send + Sync + 'static + Codec + FromStr + Display,
	AssetId: Send + Sync + 'static + Codec + FromStr + Display + Ord,
	Balance: Send + Sync + 'static + Codec + FromStr + Display,
	Error: Send + Sync + 'static + Codec + AsRef<[u8]>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: CosmwasmRuntimeApi<Block, AccountId, AssetId, Balance, Error>,
{
	fn query(
		&self,
		contract: AccountId,
		gas: u64,
		query_request: Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		let runtime_api_result = api
			.query(at, contract, gas, query_request)
			.map_err(runtime_error_into_rpc_error)?;
		runtime_api_result
			.map_err(|e| runtime_error_into_rpc_error(String::from_utf8_lossy(e.as_ref())))
	}
}
