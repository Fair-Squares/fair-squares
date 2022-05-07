#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		storage::bounded_vec::BoundedVec,
		traits::{Currency, ExistenceRequirement, Randomness},
	};
	use frame_system::pallet_prelude::*;
	// hashing algorithm for keys
	use sp_io::hashing::blake2_256;

	use codec::{Decode, Encode};
	use scale_info::TypeInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type LifeTime: Get<Self::BlockNumber>;

		//#[pallet::constant]
		//type PricePerSqm: Get<BalanceOf<T>>;

		#[pallet::constant]
		type MaxAssetOwned: Get<u32>;

		type MaxBytes: Get<u32>;
	}

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountOf<T>>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, RuntimeDebug)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Asset<T: Config> {
		owner: T::AccountId,
		location: BoundedVec<u8, T::MaxBytes>,
		sqm: u32,
		id: u32,
		onboarding_price: BalanceOf<T>,
		time_listed: T::BlockNumber,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_total_asset)]
	pub type TotalAssets<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_assets_from_owner)]
	pub type AssetMap<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<Asset<T>, T::MaxAssetOwned>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetListed(T::AccountId, u32),
		TotalAssetCount(u32),

	}

	#[pallet::error]
	pub enum Error<T> {
		MaxAssetOwnedLmit,
		MaxAssetCountLimit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn list_asset(
			owner: OriginFor<T>,
			location: BoundedVec<u8, T::MaxBytes>,
			sqm: u32,
			onboarding_amount: BalanceOf<T>,
		) -> DispatchResult {

			let onboarder = ensure_signed(owner)?;

			//incrementing id from storage TotalAssets
			let count = Self::get_total_asset().checked_add(1).ok_or(<Error<T>>::MaxAssetCountLimit)?;

			let time = <frame_system::Pallet<T>>::block_number();

			//updating TotalAssets count
			<TotalAssets<T>>::put(count);

			let asset = Asset::<T> {
				owner: onboarder.clone(),
				location: location,
				sqm: sqm,
				id: count,
				onboarding_price: onboarding_amount,
				time_listed: time,
			};

			<AssetMap<T>>::try_mutate(&onboarder, |asset_vec| asset_vec.try_push(asset))
				.map_err(|_| <Error<T>>::MaxAssetOwnedLmit)?;

			Self::deposit_event(Event::AssetListed(onboarder,count));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn get_total_assets(origin:OriginFor<T>) ->DispatchResult {
			let _ = ensure_signed(origin)?;

			let total = Self::get_total_asset();

			Self::deposit_event(Event::TotalAssetCount(total));

			Ok(())

		}

	}
}
