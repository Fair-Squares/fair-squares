#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
pub use pallet_assets as Assets;
pub use pallet_nft as Nft;
pub use pallet_roles as Roles;
pub use pallet_onboarding as Onboarding;
pub use pallet_housing_fund as HousingFund;

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
	pub trait Config: frame_system::Config + Assets::Config + Roles::Config + Nft::Config + Onboarding::Config + HousingFund::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}



	#[pallet::storage]
	#[pallet::getter(fn virtual_acc)]
	/// Stores Virtual accounts
	pub(super) type Virtual<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::NftCollectionId,
		Blake2_128Concat,
		T::NftItemId,
		Ownership<T>,
		OptionQuery,
	>;
	

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		
		/// A virtual account was created
		VirtualCreated{
			account: T::AccountId,
			collection: T::NftCollectionId,
			item: T::NftItemId,
			when: BlockNumberOf<T>
		},
		NftTransactionExecuted{
			nft_transfer_to: T::AccountId,
			nft_transfer_from: T::AccountId,
			when: BlockNumberOf<T>
		}
	}


	#[pallet::error]
	pub enum Error<T> {
		/// Not a value.
		NoneValue,
		/// This action is reserved to Accounts holding the SERVICER role.
		ReservedToServicer,
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This call creates a virtual account from the asset's collection_id and item_id.
		/// The caller must hold the Servicer role 
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_virtual(origin: OriginFor<T>, collection_id: T::NftCollectionId, item_id: T::NftItemId) -> DispatchResult {
			
			let _caller = ensure_root(origin.clone());
			let seller: T::AccountId = Nft::Pallet::<T>::owner(collection_id.clone(),item_id.clone()).unwrap();

			// Create virtual account
			Self::virtual_account(collection_id.clone(),item_id.clone()).ok();
			let account = Self::virtual_acc(collection_id.clone(),item_id.clone()).unwrap().virtual_account;

			// execute NFT transaction
			Self::nft_transaction(collection_id.clone(),item_id.clone(),account.clone()).ok();

			// Emit an event.
			let created = <frame_system::Pallet<T>>::block_number();
			Self::deposit_event(Event::VirtualCreated{
				account: account.clone(),
				collection: collection_id.clone(),
				item: item_id.clone(),
				when: created.clone(),
			});			

			//Emit another event
			Self::deposit_event(Event::NftTransactionExecuted{
				nft_transfer_to: account,
				nft_transfer_from: seller,
				when: created
			});

			Ok(())
		}

	
	}
}
