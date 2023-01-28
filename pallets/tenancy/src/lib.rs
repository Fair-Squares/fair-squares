#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet_asset_management as Assets;
pub use pallet_identity as Ident;
pub use pallet_nft as Nft;
pub use pallet_roles as Roles;
pub use pallet_share_distributor as Share;
pub use pallet_payment as Payment;

mod functions;
mod types;
pub use functions::*;
pub use types::*;

//#[cfg(feature = "runtime-benchmarks")]
//mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

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
		frame_system::Config + Assets::Config + Ident::Config + Roles::Config + Nft::Config + Payment::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}


	#[pallet::storage]
	#[pallet::getter(fn infos)]
	/// Stores Tenant informations
	pub type Tenants<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, RegisteredTenant<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		///Guaranty deposit successfully payed
		GuarantyDepositPayment{
			tenant: T::AccountId,
			when: BlockNumberOf<T>,
			asset_account: T::AccountId,
			amount: Payment::BalanceOf<T>,
		},
		///Asset Request successfully sent
		AssetRequested{
			tenant: T::AccountId,
			when: BlockNumberOf<T>,
			asset_account: T::AccountId,
		},
		///Rent payment successfully sent
		RentPayment{
			tenant: T::AccountId,
			when: BlockNumberOf<T>,
			asset_account: T::AccountId,
			amount: Roles::BalanceOf<T>,
			remaining: Roles::BalanceOf<T>,
		},
		
		
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Invalid asset id given
		NotAnAsset,
		/// The caller is not a tenant
		NotATenant,
		/// Invalid representative given
		NotARepresentative,
		/// Asset is not linked to the representative
		AssetNotLinked,
		/// The payment request is non-existant
		NotAValidPayment,
		/// The yearly rent has already been paid in full
		NoRentToPay,
		/// The tenant is not linked to the asset
		TenantAssetNotLinked,
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The function below allows an active tenant to pay for his rent.
		/// The origin must be the tenant accountId.
		/// The amount payed is the monthly_rent, and can be payed at any moment.
		/// The sum of all payments cannot exceed the yearly_rent  .
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn pay_rent(origin: OriginFor<T>) -> DispatchResult {
			let tenant_account = ensure_signed(origin.clone())?;
			let tenant = Roles::Pallet::<T>::tenants(tenant_account.clone()).unwrap();

			//Check that the Tenant is connected to the asset
			ensure!(!tenant.asset_account.clone().is_none(),Error::<T>::TenantAssetNotLinked);
			//Check that the remaining rent-to-pay is greater than 1
			ensure!(tenant.remaining_payments.clone() > 0,Error::<T>::NoRentToPay);
			//Pay the rent
			Self::rent_helper(tenant_account.clone()).ok();

			let now = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::RentPayment{
				tenant: tenant_account,
				when: now,
				asset_account: tenant.asset_account.unwrap(),
				amount: tenant.rent,
				remaining: tenant.remaining_rent,
			});

			Ok(())
		}

		/// Using the function below, a prospecting tenant can requestfor a particular asset
		/// after providing personal information requested by the Representative.
		/// The origin must be the tenant accountId.
		/// - info: Tenant personnal information requested by the asset Representative
		/// - asset_type: Asset class requested by the tenant.
		/// - asset_id: ID of the Asset requested by the tenant.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn request_asset(
			origin: OriginFor<T>,
			info: Box<IdentityInfo<T::MaxAdditionalFields>>,
			asset_type: Nft::PossibleCollections,
			asset_id: T::NftItemId,
		) -> DispatchResult {
			let caller = ensure_signed(origin.clone())?;
		// Ensure that the caller has the tenancy role
		ensure!(Roles::TenantLog::<T>::contains_key(caller.clone()), Error::<T>::NotATenant);

		// Ensure that the asset is valid
		let collection_id: T::NftCollectionId = asset_type.value().into();
		let ownership = Share::Pallet::<T>::virtual_acc(collection_id, asset_id);
		ensure!(ownership.is_some(), Error::<T>::NotAnAsset);
		let virtual_account = ownership.unwrap().virtual_account;
		Self::request_helper(origin.clone(),virtual_account.clone(),info).ok();
		let now = <frame_system::Pallet<T>>::block_number();

		Self::deposit_event(Event::AssetRequested{
			tenant: caller,
			when: now,
			asset_account: virtual_account,
		});
		


		Ok(())

		}

		/// The function below allows the newly selected tenant to pay for a guaranty deposit request
		/// and confirm the start of his contract.
		/// The origin must be the tenant.
		/// - asset_type: Asset class requested by the tenant.
		/// - asset_id: ID of the Asset requested by the tenant.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn pay_guaranty_deposit(
		origin: OriginFor<T>,
		asset_type: Nft::PossibleCollections,
		asset_id: T::NftItemId,
	) -> DispatchResult {
		let caller = ensure_signed(origin.clone())?;
		// Ensure that the caller has the tenancy role
		ensure!(Roles::TenantLog::<T>::contains_key(&caller), Error::<T>::NotATenant);

		// Ensure that the asset is valid
		let collection_id: T::NftCollectionId = asset_type.value().into();
		let ownership = Share::Pallet::<T>::virtual_acc(collection_id, asset_id);
		ensure!(ownership.is_some(), Error::<T>::NotAnAsset);
		let virtual_account = ownership.unwrap().virtual_account;

		//Ensure that payment request exists
		ensure!(Assets::GuarantyPayment::<T>::contains_key(&caller,&virtual_account),Error::<T>::NotAValidPayment);
		let payment_infos = Assets::Pallet::<T>::guaranty(&caller,&virtual_account).unwrap();
		let status = payment_infos.state;
		ensure!(status == Payment::PaymentState::PaymentRequested,Error::<T>::NotAValidPayment);

		Self::payment_helper(origin,virtual_account.clone(),collection_id,asset_id).ok();
		let now = <frame_system::Pallet<T>>::block_number();

		Self::deposit_event(Event::GuarantyDepositPayment{
			tenant: caller,
			when: now,
			asset_account: virtual_account,
			amount: payment_infos.amount
		});
		

		Ok(())
	}


	}

	
}
