#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
///Kazu:Importing needed types from the NFT pallet
type ClassData<T> = <T as orml_nft::Config>::ClassData;
type TokenData<T> = <T as orml_nft::Config>::TokenData;
type TokenId<T> = <T as orml_nft::Config>::TokenId;
type ClassId<T> = <T as orml_nft::Config>::ClassId;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResult,
		ensure,
		pallet_prelude::*,
		sp_runtime::traits::{AccountIdConversion, Hash, Saturating, Zero},
		storage::child,
		traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons},
		PalletId
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::TypeInfo;
	use frame_support::inherent::Vec;
	use scale_info::prelude::vec;

	const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
	const TreasurePalletId: PalletId = PalletId(*b"py/trsry");
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config  {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionDeposit: Get<BalanceOf<Self>>;
		type MinContribution: Get<BalanceOf<Self>>;
		// type RetirementPeriod: Get<Self::BlockNumber>;
	}



	///Kazu:the struct below is used for project's proposal's: Houses, Business, land, etc..
	///Kazu:the proposal is linked to a NFT which represents the proposal contract 
	#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Proposal<AccountId, Balance,ClassId, TokenId> {
		/// Kazu:The account that will receive the funds if the proposal is accepted.
		powner: AccountId,
		price: Balance,
		classId: ClassId,
		tokenId: TokenId,
	}

	///Kazu:The struct below is used to track contributors & contributions
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
	pub struct ContrIb<AccountId, Balance> {
		contribution: Balance,
		account: AccountId,
}

	
	pub type PropIndex = u32;//Kazu:for proposals	
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type ContIndex<T> = Vec<AccountIdOf<T>>;//Kazu:nbr of contributors
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	type ProposalInfoOf<T> = Proposal<AccountIdOf<T>, BalanceOf<T>,ClassId<T>, TokenId<T>>; //Kazu:for proposals


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	///Kazu:Below is the definition of the storage used for contributions
	#[pallet::storage]
	#[pallet::getter(fn contr_ib)]
	pub(super) type ContStore<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn props)]
	/// Kazu:Info on all of the proposals.
	pub(super) type Props<T: Config> =
		StorageMap<_, Blake2_128Concat,PropIndex, ProposalInfoOf<T>, OptionQuery>;

	

	#[pallet::storage]
	#[pallet::getter(fn prop_count)]
	/// Kazu:The total number of proposals that have so far been submitted.
	pub(super) type PropCount<T: Config> = StorageValue<_, PropIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cont_count)]
	/// Kazu:The total number of proposals that have so far been submitted.
	pub(super) type ContAcc<T: Config> = StorageValue<_, ContIndex<T>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	// #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance",T::BlockNumber = "BlockNumber")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		Created( <T as frame_system::Config>::BlockNumber),
		Created2(PropIndex, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
		Contributed(
			<T as frame_system::Config>::AccountId,
			BalanceOf<T>,
			<T as frame_system::Config>::BlockNumber,
		),
		Withdrew(
			<T as frame_system::Config>::AccountId,
			BalanceOf<T>,
			<T as frame_system::Config>::BlockNumber,
		),
		Retiring(<T as frame_system::Config>::BlockNumber),
		Dissolved(
			
			<T as frame_system::Config>::BlockNumber,
			<T as frame_system::Config>::AccountId,
		),
		Dispensed(
			
			<T as frame_system::Config>::BlockNumber,
			<T as frame_system::Config>::AccountId,
		),
	}



	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Crowdfund must end after it starts
		EndTooEarly,
		/// Must contribute at least the minimum amount of funds
		ContributionTooSmall,
		/// The fund index specified does not exist
		InvalidIndex,
		/// The crowdfund's contribution period has ended; no more contributions will be accepted
		ContributionPeriodOver,
		/// You may not withdraw or dispense funds while the fund is still active
		FundStillActive,
		/// You cannot withdraw funds because you have not contributed any
		NoContribution,
		/// You cannot dissolve a fund that has not yet completed its retirement period
		FundNotRetired,
		/// Cannot dispense funds from an unsuccessful fund
		UnsuccessfulFund,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Kazu:Create a new Proposal
		#[pallet::weight(10_000)]
		pub fn createProp(
			origin: OriginFor<T>,
			powner: AccountIdOf<T>,
			price: BalanceOf<T>,
			_Cdatas:ClassData<T>, //Kazu: Added ClassData parameter from orml_nft pallet
			_Tdatas:TokenData<T> //Kazu: Added TokenData parameter from orml_nft pallet
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			
			let deposit = T::SubmissionDeposit::get();
			let imb = T::Currency::withdraw(
				&creator,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;
			//Kazu: I need to understand what goes in metadata and data parameters. For now I use a dummy vector for metada, and I create a NFT
			let vv = vec![3,5];
			let vv2 = vv.clone();
			//Kazu:Creating the nftClassId and the tokenId(Minting)
			let classId = orml_nft::Pallet::<T>::create_class(&powner,vv,Default::default())?;
			let tokenId= orml_nft::Pallet::<T>::mint(&powner,classId,vv2,Default::default())?;
			
			let index = <PropCount<T>>::get();
			// not protected against overflow, see safemath section
			<PropCount<T>>::put(index + 1);
			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			T::Currency::resolve_creating(&TreasurePalletId.into_account(), imb);

			//Kazu:Storing the created proposal informations inside the Props storage
			<Props<T>>::insert(
				index,
				Proposal { powner,price,classId,tokenId},
			);

			Self::deposit_event(Event::Created( now));
			Ok(().into())
		}

		/// Contribute funds to an existing fund
		#[pallet::weight(10_000)]
		pub fn contribute(
			origin: OriginFor<T>,
			account: T::AccountId,//Kazu:Added this for compatibility with storageMap
			//to: T::AccountId,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {

		///Kazu:creating a new contribution object below with the current infos
			let c1= self::ContrIb{
				contribution: value,
				account: &account,
			};
			let who = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			///Kazu:if id is already in storage, update storage value by adding the new contribution, or else
			///insert the new Id/contribution
			if ContStore::<T>::contains_key(c1.account){
				ContStore::<T>::mutate(c1.account,|value|{
					*value += c1.contribution;
				})

			} else {
				ContStore::<T>::insert(&account,value);
				//let mut ve=<ContAcc<T>>::get();
				ContAcc::<T>::mutate(|val|{
					val.push(account);
				})				

				}
			
			ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);

			// Add contribution to the fund
			T::Currency::transfer(
				&who,
				&TreasurePalletId.into_account(),
				value,
				ExistenceRequirement::AllowDeath,
			)?;
			
			

			let balance = Self::contribution_get(&who);
			let balance = balance.saturating_add(value);
			Self::contribution_put(&who, &balance);

			Self::deposit_event(Event::Contributed(who, balance, now));

			Ok(().into())
		}


		///Proposal transactions
		#[pallet::weight(10_000)]
		pub fn fundProp(
		//Kazu: Origin is the account paying for transaction fees.
			_origin: OriginFor<T>,
			index1: PropIndex,
			
						
		)-> DispatchResultWithPostInfo{
			//Kazu: Pay the proposal owner From Treasurery
			
			let prop = Props::<T>::get(index1).ok_or(Error::<T>::InvalidIndex)?;
			
			let price = prop.price;
			let powner = prop.powner;
			let ben = TreasurePalletId.into_account();
			
			T::Currency::transfer(
				&ben,
				&powner,
				price,
				ExistenceRequirement::AllowDeath,
			)?;
			
			//Determine which contributor is included into the proposal
			//Creating a vector containing contributor's IDs
			let mut ve=<ContAcc<T>>::get();
			//For each contributor ID
			for i in ve.iter(){
				//We get the total of [raised funds]~[Treasury]
				
				let total = Self::pot();

				//pick-up 1st contribution and convert it from type Balance to u64
				let contrib = Self::contr_ib(i);
				let mut contrib1 = TryInto::<u64>::try_into(contrib).ok();

				//pick-up proposal price and convert it from type Balance to u64
				let mut pr = TryInto::<u64>::try_into(price).ok();
				let pric= match pr{
					Some(x) => x,
					None => 0,
				};
				//contrib1 is an enum collection, so we use match to extract the contribution
				let b0= match contrib1{
					Some(x) => x,
					None => 0,
				};
				
				//convert the raised funds from Balance to u64,
				//and then extract the value from the enum collection 
				let mut total0 = TryInto::<u64>::try_into(total).ok();
				let b1= match total0{
					Some(x) => x,
					None => 0,
				};
				//In order to use divisions, we need both values to be floats
				let price2 = pric as f64;
				let b00= b0 as f64;
				let b11= b1 as f64;
				//contribution percentage calculation 
				let mut per = 100.0*(b00/b11);
				//amount removed from contributor share
				let newcon=(&per*price2/100.0) as u8;

				let perc = per as u8;

				let newb=TryInto::<BalanceOf<T>>::try_into(newcon).ok();
				let b= match newb {
					Some(x) => x,
					None => Zero::zero(),
				};
				//We need to update the contribution storage
				ContStore::<T>::mutate(i,|value|{
					
					*value-= b;
				});

				
				let classId = orml_nft::Pallet::<T>::transfer(&powner,&i,(prop.classId,prop.tokenId),perc);
			}
		
			//calculate purcentage based on number of contributors
		
			//distribute NFTs to contributors
			Ok(().into())		
		}

		/// Withdraw full balance of a contributor to treasury
		#[pallet::weight(10_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			#[pallet::compact]index: PropIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut fund = Self::props(index).ok_or(Error::<T>::InvalidIndex)?;
			let now = <frame_system::Pallet<T>>::block_number();
			// ensure!(fund.end < now, Error::<T>::FundStillActive);

			let balance = Self::contribution_get(&who);
			ensure!(balance > Zero::zero(), Error::<T>::NoContribution);

			// Return funds to caller without charging a transfer fee
			let _ = T::Currency::resolve_into_existing(
				&who,
				T::Currency::withdraw(
					&TreasurePalletId.into_account(),
					balance,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Update storage
			Self::contribution_kill( &who);
			Self::deposit_event(Event::Withdrew(who, balance, now));

			Ok(().into())
		}


		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {

		/// Find the ID associated with the fund
		///
		/// Each fund stores information about its contributors and their contributions in a child trie
		/// This helper function calculates the id of the associated child trie.
		pub fn id_from_index() -> child::ChildInfo {
			let mut buf = Vec::new();
			buf.extend_from_slice(b"treasury");
			//buf.extend_from_slice(&index.to_le_bytes()[..]);

			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}

		/// Record a contribution in the associated child trie.
		pub fn contribution_put( who: &T::AccountId, balance: &BalanceOf<T>) {
			let id = Self::id_from_index();
			who.using_encoded(|b| child::put(&id, b, &balance));
		}

		/// Lookup a contribution in the associated child trie.
		pub fn contribution_get(who: &T::AccountId) -> BalanceOf<T> {
			let id = Self::id_from_index();
			who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
		}

		/// Remove a contribution from an associated child trie.
		pub fn contribution_kill(who: &T::AccountId) {
			let id = Self::id_from_index();
			who.using_encoded(|b| child::kill(&id, b));
		}

		/// Remove the entire record of contributions in the associated child trie in a single
		/// storage write.
		pub fn crowdfund_kill() {
			let id = Self::id_from_index();
			// The None here means we aren't setting a limit to how many keys to delete.
			// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
			// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
			child::kill_storage(&id, None);
		}

		pub fn pot() -> BalanceOf<T> {
			T::Currency::free_balance(&TreasurePalletId.into_account())
			// Must never be less than 0 but better be safe.
			.saturating_sub(T::Currency::minimum_balance())
	}
		//pub fn account_id() -> T::AccountId {
		//T::PalletId::get().into_account()
	//}
}
	}

