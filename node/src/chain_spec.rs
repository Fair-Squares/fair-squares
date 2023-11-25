use fs_node_runtime::{
	pallet_roles,RolesModuleConfig,AccountId, OnboardingModuleConfig,AuraConfig, BalancesConfig, NftModuleConfig,CouncilConfig,BackgroundCouncilConfig,RuntimeGenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY,
};

use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{traits::{IdentifyAccount, Verify}, AccountId32};
use hex_literal::hex;

// The URL for the telemetry server.
 const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
 const META: Vec<u8> = vec![];
 const REP0: Vec<AccountId32> = vec![];


/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

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

pub fn get_endowed_accounts_with_balance() -> Vec<(AccountId, u128)> {
	let accounts: Vec<AccountId> = vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
	];

	let accounts_with_balance: Vec<(AccountId, u128)> =
		accounts.iter().cloned().map(|k| (k, 1 << 60)).collect();
	let json_data = &include_bytes!("../../seed/balances.json")[..];
	let additional_accounts_with_balance: Vec<(AccountId, u128)> =
		serde_json::from_slice(json_data).unwrap();

	let mut accounts = additional_accounts_with_balance.clone();

	accounts_with_balance.iter().for_each(|tup1| {
		for tup2 in additional_accounts_with_balance.iter() {
			if tup1.0 == tup2.0 {
				return
			}
		}
		accounts.push(tup1.to_owned());
	});

	accounts
}

fn testnet_endowed_accounts() -> Vec<(AccountId,u128)> {
	let accounts: Vec<AccountId> = vec![
		//5CcZ9b6Wezpos7BBfcN3o4Jam6NQ8tU4gb4Xa7roMCCyND5X
		hex!["184a89cbb6aa857b41c98841be365ab3947ef1f729aa6fe0f6a1322f6391945b"].into(),

		//5D1nJ8M862utBuDG7mwszhm1o7Tzn5FSd6BYEoX3uc5e3AuB
		hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),

		//5FnsQ9tHAjqbb1YHU4hAr2hPgDS82XGvNPA35CpV7hX58sVv ==> Faucet
		hex!["a4dd21a5be2a4d85b80a0091f77b3371fa3c4f3cf511bead59cf65583251ce16"].into(),

		//5Ferj4SHg8mtUGxWHuyjhciTCmh7TAyhw8pSjFtxRAJDynpk 
		hex!["9ec0e63219270075ffd546e4fa39b4027216a9de5ed16b38bc54d66fe09b8d47"].into(),
	];
	let accounts_with_balance: Vec<(AccountId, u128)> =
		accounts.iter().cloned().map(|k| (k, 1 << 60)).collect();
	let json_data = &include_bytes!("../../seed/balances.json")[..];
	let additional_accounts_with_balance: Vec<(AccountId, u128)> =
		serde_json::from_slice(json_data).unwrap();

	let mut accounts = additional_accounts_with_balance.clone();

	accounts_with_balance.iter().for_each(|tup1| {
		for tup2 in additional_accounts_with_balance.iter() {
			if tup1.0 == tup2.0 {
				return;
			}
		}
		accounts.push(tup1.to_owned());
	});

	accounts
}

pub fn fs_properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "FST".into());
	properties
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		],
		true,
	))
	.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		true,
	))
	.build())
}

pub fn square_one_testnet() -> Result<ChainSpec, String> {

	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		],
		true,
	))
	.build())
}


fn square_one(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({

		"system": {},
		"balances": {
			// Configure endowed accounts with initial balance of 1 << 60.
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"aura": {
			"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		"grandpa": {
			"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key.clone()),
		},
		

		"roles_module": {
			"new_admin": Some(AccountId32::from(hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"])),
			"representatives": REP0,
		},
		"nft_module": {
			"owner": Some(root_key.clone()),
			"collection_id": Some(0),
			"created_by": Some(pallet_roles::Accounts::SERVICER),
			"metadata": META,
		},
		"onboarding_module":{
			"root":Some(root_key),
		},
		//"democracy": Default::default(),
		//"treasury": Default::default(),
		"council": {
			"members": vec![
				AccountId32::from(hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"]),
				AccountId32::from(hex!["184a89cbb6aa857b41c98841be365ab3947ef1f729aa6fe0f6a1322f6391945b"]),
				AccountId32::from(hex!["9ec0e63219270075ffd546e4fa39b4027216a9de5ed16b38bc54d66fe09b8d47"]),
			],
			//"phantom": Default::default(),
		},
		"background_council": {
			"members": vec![
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			],
			//"phantom": Default::default(),
		},
		//"assets": Default::default()
	})
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({

		"system": {},
		"balances": {
			// Configure endowed accounts with initial balance of 1 << 60.
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"aura": {
			"authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
		},
		"grandpa": {
			"authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		"sudo": {
			// Assign network admin rights.
			"key": Some(root_key.clone()),
		},
		

		"roles_module": {
			"new_admin": Some(AccountId32::from(hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"])),
			"representatives": REP0,
		},
		"nft_module": {
			"owner": Some(root_key.clone()),
			"collection_id": Some(0),
			"created_by": Some(pallet_roles::Accounts::SERVICER),
			"metadata": META,
		},
		"onboarding_module":{
			"root":Some(root_key),
		},
		//"democracy": Default::default(),
		//"treasury": Default::default(),
		"council": {
			"members": vec![
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			],
			//"phantom": Default::default(),
		},
		"background_council": {
			"members": vec![
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
			],
			//"phantom": Default::default(),
		},
		//"assets": Default::default()
	})
}
