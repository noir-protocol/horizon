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

use horizon_template_runtime::{AccountId, RuntimeGenesisConfig, Signature, WASM_BINARY};
use sc_chain_spec::NoExtension;
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ecdsa, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec =
	sc_service::GenericChainSpec<RuntimeGenesisConfig, NoExtension, hp_io::crypto::HostFunctions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::builder(wasm_binary, None)
		.with_name("Development")
		.with_id("dev")
		.with_chain_type(ChainType::Development)
		.with_genesis_config_patch(testnet_genesis(
			// Initial PoA authorities
			vec![authority_keys_from_seed("Alice")],
			// Sudo account
			get_account_id_from_seed::<ecdsa::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<ecdsa::Public>("Alice"),
				get_account_id_from_seed::<ecdsa::Public>("Bob"),
				get_account_id_from_seed::<ecdsa::Public>("Alice//stash"),
				get_account_id_from_seed::<ecdsa::Public>("Bob//stash"),
			],
			true,
		))
		.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::builder(wasm_binary, None)
		.with_name("Local Testnet")
		.with_id("local_testnet")
		.with_chain_type(ChainType::Local)
		.with_genesis_config_patch(testnet_genesis(
			// Initial PoA authorities
			vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
			// Sudo account
			get_account_id_from_seed::<ecdsa::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<ecdsa::Public>("Alice"),
				get_account_id_from_seed::<ecdsa::Public>("Bob"),
				get_account_id_from_seed::<ecdsa::Public>("Alice//stash"),
				get_account_id_from_seed::<ecdsa::Public>("Bob//stash"),
			],
			true,
		))
		.build())
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"cosmosAccounts": {
			"accounts": endowed_accounts.to_vec(),
		},
		"aura": {
			"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		"grandpa": {
			"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key),
		},
	})
}
