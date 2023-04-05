use fs_node_runtime::{
	pallet_roles, AccountId, AuraConfig, BalancesConfig, CouncilConfig, GenesisConfig,
	GrandpaConfig, NftModuleConfig, RoleModuleConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;


// The URL for the telemetry server.
const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";


/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
		// All the accounts below have to be manually added to Polkadot.JS using the secret seed
		// obtained with for example:  `target/release/fs-node key inspect //KillMonger`
		// The account balance can then be customised, using the `/seed/balances.json` file
		get_account_id_from_seed::<sr25519::Public>("KillMonger"),
		get_account_id_from_seed::<sr25519::Public>("Aluman"),
		get_account_id_from_seed::<sr25519::Public>("Shikamaru"),
		get_account_id_from_seed::<sr25519::Public>("Geraldo"),
		get_account_id_from_seed::<sr25519::Public>("Gabriel"),
		get_account_id_from_seed::<sr25519::Public>("Henry"),
		get_account_id_from_seed::<sr25519::Public>("Hans"),
		get_account_id_from_seed::<sr25519::Public>("Obito"),
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
		hex!["184a89cbb6aa857b41c98841be365ab3947ef1f729aa6fe0f6a1322f6391945b"].into(),
		hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
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
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				// Pre-funded accounts
				get_endowed_accounts_with_balance(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(fs_properties()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				// Pre-funded accounts
				get_endowed_accounts_with_balance(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(fs_properties()),
		// Extensions
		None,
	))
}

pub fn square_one_testnet() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Fair Squares testnet",
		// ID
		"square-one",
		ChainType::Live,
		move || {
			square_one(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				// Pre-funded accounts
				testnet_endowed_accounts(),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		None,
		None,
		// Properties
		Some(fs_properties()),
		// Extensions
		None,
	))
}


/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts,
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		transaction_payment: Default::default(),

		role_module: RoleModuleConfig {
			new_admin: Some(hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into()),
			representatives: vec![
			],
		},
		nft_module: NftModuleConfig {
			owner: Some(root_key),
			collection_id: Some(3),
			created_by: Some(pallet_roles::Accounts::SERVICER),
			metadata: Some(b"metadata".to_vec().try_into().unwrap()),
		},
		democracy: Default::default(),
		treasury: Default::default(),
		council: CouncilConfig {
			members: vec![
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				hex!["184a89cbb6aa857b41c98841be365ab3947ef1f729aa6fe0f6a1322f6391945b"].into(),
				hex!["9ec0e63219270075ffd546e4fa39b4027216a9de5ed16b38bc54d66fe09b8d47"].into(),
			],
			phantom: Default::default(),
		},
		assets: Default::default(),
	}
}


fn square_one(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts,
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		transaction_payment: Default::default(),

		role_module: RoleModuleConfig {
			new_admin: Some(hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into()),
			representatives: vec![
			],
		},
		nft_module: NftModuleConfig {
			owner: Some(root_key),
			collection_id: Some(1),
			created_by: Some(pallet_roles::Accounts::SERVICER),
			metadata: Some(b"metadata".to_vec().try_into().unwrap()),
		},
		democracy: Default::default(),
		treasury: Default::default(),
		council: CouncilConfig {
			members: vec![
				hex!["2a0170a78af6835dd46753c1857b31903aa125d9c203e05bc7a45b7c3bea702b"].into(),
				hex!["184a89cbb6aa857b41c98841be365ab3947ef1f729aa6fe0f6a1322f6391945b"].into(),
				hex!["9ec0e63219270075ffd546e4fa39b4027216a9de5ed16b38bc54d66fe09b8d47"].into(),
			],
			phantom: Default::default(),
		},
		assets: Default::default(),
	}
}