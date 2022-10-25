//! # Share_Distributor Pallet
//!
//! The Share_Distributor Pallet is called by the Bidding Pallet
//! after a Finalised bid was identified on-chain.
//! It will distribute to the asset new owners, the ownership nft, and the ownership token
//! connected to the asset at the center of the transaction.
//!
//! ## Overview
//!
//! The Share_Distributor Pallet fulfill the following tasks:
//! - Create a virtual account which will hold the nft
//! - Connect the Virtual account to the new owners/contributors
//! through the use of a storage/struct
//! - Execute the Nft transaction between Seller and Virtual account
//! - Mint 1000 Ownership Tokens, which represent the asset share of
//! each owner
//! - Distribute the ownership tokens to the new owners.
//!
//! Dispatchable Functions
//!
//! * `create_virtual` - Will sequencially execute each of the steps
//! described in the Overview.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use pallet_assets as Assets;
pub use pallet_housing_fund as HousingFund;
pub use pallet_nft as Nft;
pub use pallet_onboarding as Onboarding;
pub use pallet_roles as Roles;

mod functions;
mod types;
pub use functions::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ Assets::Config
		+ Roles::Config
		+ Nft::Config
		+ Onboarding::Config
		+ HousingFund::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type AssetId: IsType<<Self as Assets::Config>::AssetId> + Parameter + From<u32> + Ord + Copy;
		#[pallet::constant]
		type Fees: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn virtual_acc)]
	/// Stores Virtual accounts
	pub type Virtual<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		Ownership<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn tokens_infos)]
	/// Stores Tokens infos
	pub type Tokens<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Owners<T>, OptionQuery>;

	#[pallet::type_value]
	///Initializing Token id to value 0
	pub fn InitDefault<T: Config>() -> u32 {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn token_id)]
	/// Stores Ownership Tokens id number
	pub type TokenId<T: Config> = StorageValue<_, u32, ValueQuery, InitDefault<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A virtual account was created
		VirtualCreated {
			account: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			when: BlockNumberOf<T>,
		},
		NftTransactionExecuted {
			nft_transfer_to: T::AccountId,
			nft_transfer_from: T::AccountId,
			when: BlockNumberOf<T>,
		},
		OwnershipTokensDistributed {
			from: T::AccountId,
			to: Vec<T::AccountId>,
			token_id: <T as pallet::Config>::AssetId,
			owners: Vec<(T::AccountId, <T as Assets::Config>::Balance)>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not a value.
		NoneValue,
		/// Ivalid parameter
		InvalidValue,
		/// This action is reserved to Accounts holding the SERVICER role.
		ReservedToServicer,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This call creates a virtual account from the asset's collection_id and item_id.
		/// The caller must hold the Servicer role
		#[pallet::weight(10_000)]
		pub fn create_virtual(
			origin: OriginFor<T>,
			collection_id: T::NftCollectionId,
			item_id: T::NftItemId,
		) -> DispatchResult {
			let _caller = ensure_root(origin.clone());
			let seller: T::AccountId = Nft::Pallet::<T>::owner(collection_id, item_id).unwrap();
			// Create virtual account
			Self::virtual_account(collection_id, item_id).ok();
			let account = Self::virtual_acc(collection_id, item_id).unwrap().virtual_account;

			// execute NFT transaction
			Self::nft_transaction(collection_id, item_id, account.clone()).ok();

			//Create new token class
			Self::create_tokens(origin, collection_id, item_id, account.clone()).ok();

			//distribute tokens
			Self::distribute_tokens(account.clone(), collection_id, item_id).ok();

			// Update Housing fund informations
			HousingFund::Pallet::<T>::validate_house_bidding(collection_id, item_id).ok();

			// Emit some events.
			let created = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::VirtualCreated {
				account: account.clone(),
				collection: collection_id,
				item: item_id,
				when: created,
			});

			Self::deposit_event(Event::NftTransactionExecuted {
				nft_transfer_to: account.clone(),
				nft_transfer_from: seller,
				when: created,
			});

			let new_owners = Self::virtual_acc(collection_id, item_id).unwrap().owners;
			let token_id = Self::virtual_acc(collection_id, item_id).unwrap().token_id;
			let owners = Self::tokens_infos(account.clone()).unwrap().owners;
			Self::deposit_event(Event::OwnershipTokensDistributed {
				from: account,
				to: new_owners,
				token_id,
				owners,
			});

			Ok(())
		}
	}
}
