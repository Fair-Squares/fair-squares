
// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
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

//#[cfg(test)]
//mod mock;

//#[cfg(test)]
//mod tests;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;
//pub mod weights;
//pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);


	#[pallet::config]
	pub trait Config: 
		frame_system::Config
		+ Assets::Config
		+ Roles::Config
		+ Nft::Config
		+ Onboarding::Config
		+ HousingFund::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetId: 
		IsType<<Self as Assets::Config>::AssetIdParameter> 
		+ Into<<Self as Assets::Config>::AssetId>
		+ Parameter 
		+ From<u32> 
		+ Ord 
		+ Copy
		+MaxEncodedLen;
		#[pallet::constant]
		type Fees: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type MaxOwners: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

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
		/// A user has successfully set a new value.
		SomethingStored {
			/// The new value set.
			something: u32,
			/// The account who set the new value.
			who: T::AccountId,
		},
		/// A virtual account was created
		VirtualCreated {
			account: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			when: BlockNumberOf<T>,
		},
		/// NFT Transaction Executed
		NftTransactionExecuted {
			nft_transfer_to: T::AccountId,
			nft_transfer_from: T::AccountId,
			when: BlockNumberOf<T>,
		},
		///Ownership Token distributed to new owners 
		OwnershipTokensDistributed {
			from: T::AccountId,
			to: BoundedVec<T::AccountId,T::MaxOwners>,
			token_id: <T as pallet::Config>::AssetId,
			owners: BoundedVec<(T::AccountId, <T as Assets::Config>::Balance),T::MaxOwners>,
		},
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
		/// Not a value.
		NoneValue,
		/// Ivalid parameter
		InvalidValue,
		/// This action is reserved to Accounts holding the SERVICER role.
		ReservedToServicer,
		/// Not enough funds in the fees_account
		NotEnoughFees,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a single u32 value as a parameter, writes the value
		/// to storage and emits an event.
		///
		/// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
		/// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
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