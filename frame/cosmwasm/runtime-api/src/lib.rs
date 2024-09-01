#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

extern crate alloc;

use alloc::vec::Vec;
use codec::Codec;

// Cosmwasm Runtime API declaration.
sp_api::decl_runtime_apis! {
	pub trait CosmwasmRuntimeApi<AccountId, AssetId, Balance, Error>
	where
		AccountId: Codec,
		AssetId: Codec,
		Balance: Codec,
		Error: Codec
	{
		fn query(
			contract: AccountId,
			gas: u64,
			query_request: Vec<u8>,
		) -> Result<Vec<u8>, Error>;
	}
}
