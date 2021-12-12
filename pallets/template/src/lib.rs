#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
///Kazu:Importing needed types from the NFT pallet
type ClassData<T> = <T as orml_nft::Config>::ClassData;
type TokenData<T> = <T as orml_nft::Config>::TokenData;
type TokenId<T> = <T as orml_nft::Config>::TokenId;
type ClassId<T> = <T as orml_nft::Config>::ClassId;
type index = u32;
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

	#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct FundInfo<AccountId, Balance> {
		/// The account that will recieve the funds if the campaign is successful.
		beneficiary: AccountId,
		/// The amount of deposit placed.
		deposit: Balance,
		/// The total amount raised.
		raised: Balance,
		
	}

	///Kazu:the struct below is used for project's proposal's: Houses, Business, land, etc..
	///Kazu:the proposal is linked to a NFT which represents the proposal contract 
	#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Proposal<AccountId, Balance,ClassId, TokenId,index> {
		/// Kazu:The account that will receive the funds if the proposal is accepted.
		powner: AccountId,
		price: Balance,
		classId: ClassId,
		tokenId: TokenId,
		index:index,
	}

	///Kazu:The struct below is used to track contributors & contributions
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
	pub struct ContrIb<AccountId, Balance> {
		contribution: Balance,
		account: AccountId,
		
}

	pub type FundIndex = u32;
	pub type PropIndex = u32;//Kazu:for proposals
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//type ProposalInfoOf<T> = Proposal<AccountIdOf<T>, BalanceOf<T>,ClassId<T>, TokenId<T>>; //Kazu:for proposals
	type FundInfoOf<T> = 
		FundInfo<AccountIdOf<T>, BalanceOf<T>>;

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
	#[pallet::getter(fn funds)]
	/// Info on all of the funds.
	pub(super) type Funds<T: Config> =
		StorageMap<_, Blake2_128Concat, FundIndex, FundInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn props)]
	/// Kazu:Info on all of the proposals.
	pub(super) type Props<T: Config> =
		StorageValue<_, Proposal<AccountIdOf<T>, BalanceOf<T>,ClassId<T>, TokenId<T>,index>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	/// The total number of funds that have so far been allocated.
	pub(super) type FundCount<T: Config> = StorageValue<_, FundIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn prop_count)]
	/// Kazu:The total number of proposals that have so far been submitted.
	pub(super) type PropCount<T: Config> = StorageValue<_, PropIndex, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	// #[pallet::metadata(T::AccountId = "AccountId", T::Balance = "Balance",T::BlockNumber = "BlockNumber")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		Created(FundIndex, <T as frame_system::Config>::BlockNumber),
		Created2(PropIndex, <T as frame_system::Config>::BlockNumber), //Kazu:for creation of a proposal
		Contributed(
			<T as frame_system::Config>::AccountId,
			FundIndex,
			BalanceOf<T>,
			<T as frame_system::Config>::BlockNumber,
		),
		Withdrew(
			<T as frame_system::Config>::AccountId,
			FundIndex,
			BalanceOf<T>,
			<T as frame_system::Config>::BlockNumber,
		),
		Retiring(FundIndex, <T as frame_system::Config>::BlockNumber),
		Dissolved(
			FundIndex,
			<T as frame_system::Config>::BlockNumber,
			<T as frame_system::Config>::AccountId,
		),
		Dispensed(
			FundIndex,
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
		/// Create a new fund
		#[pallet::weight(10_000)]
		pub fn create(
			origin: OriginFor<T>,
			// beneficiary: AccountIdOf<T>,
			// goal: BalanceOf<T>,
			// end: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			// ensure!(end > now, Error::<T>::EndTooEarly);
			let deposit = T::SubmissionDeposit::get();
			let beneficiary = TreasurePalletId.into_account();
			let imb = T::Currency::withdraw(
				&creator,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			let index = <FundCount<T>>::get();
			// not protected against overflow, see safemath section
			<FundCount<T>>::put(index + 1);
			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			T::Currency::resolve_creating(&Self::fund_account_id(index), imb);

			<Funds<T>>::insert(
				index,
				FundInfo { beneficiary, deposit, raised: Zero::zero() },
			);

			Self::deposit_event(Event::Created2(index, now));
			Ok(().into())
		}

		/// Kazu:Create a new Proposal
		#[pallet::weight(10_000)]
		pub fn createProp(
			origin: OriginFor<T>,
			powner: AccountIdOf<T>,
			price: BalanceOf<T>,
			Cdatas:ClassData<T>, //Kazu: Added ClassData parameter from orml_nft pallet
			Tdatas:TokenData<T> //Kazu: Added TokenData parameter from orml_nft pallet
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
			let mut vv = vec![3,5];
			let mut vv2 = vv.clone();
			//Kazu:Creating the nftClassId and the tokenId(Minting)
			let classId = orml_nft::Pallet::<T>::create_class(&powner,vv,Default::default())?;
			let tokenId= orml_nft::Pallet::<T>::mint(&powner,classId,vv2,Default::default())?;
			
			let index = <PropCount<T>>::get();
			// not protected against overflow, see safemath section
			<PropCount<T>>::put(index + 1);
			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			T::Currency::resolve_creating(&Self::fund_account_id(index), imb);

			//Kazu:Storing the created proposal informations inside the Props storage
			//<Props<T>>::insert(
			//	index,
			//	Proposal { powner,price,classId,tokenId},
			//);
			
			let mut pro=Props::<T>::get();
			pro.powner=powner;
			pro.price=price;
			pro.classId=classId;
			pro.tokenId=tokenId;
			pro.index=index;

			Self::deposit_event(Event::Created(index, now));
			Ok(().into())
		}

		/// Contribute funds to an existing fund
		#[pallet::weight(10_000)]
		pub fn contribute(
			origin: OriginFor<T>,
			account: T::AccountId,//Kazu:Added this for compatibility with storageMap
			index: FundIndex,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {

		///Kazu:creating a new contribution object below with the current infos
			let c1= self::ContrIb{
				contribution: value,
				account: &account,

			};
			let who = ensure_signed(origin)?;

			///Kazu:if id is already in storage, update storage value by adding the new contribution, or else
			///insert the new Id/contribution
			if ContStore::<T>::contains_key(c1.account){
				ContStore::<T>::mutate(c1.account,|value|{
					*value += c1.contribution;
				})
			} else {ContStore::<T>::insert(account,value);}
			
			ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;
			
			// Make sure crowdfund has not ended
			let now = <frame_system::Pallet<T>>::block_number();
			// ensure!(fund.end > now, Error::<T>::ContributionPeriodOver);

			// Add contribution to the fund
			T::Currency::transfer(
				&who,
				&Self::fund_account_id(index),
				value,
				ExistenceRequirement::AllowDeath,
			)?;
			fund.raised += value;
			Funds::<T>::insert(index, &fund);

			let balance = Self::contribution_get(index, &who);
			let balance = balance.saturating_add(value);
			Self::contribution_put(index, &who, &balance);

			Self::deposit_event(Event::Contributed(who, index, balance, now));

			Ok(().into())
		}


		///Kazu:Proposal funding, and NFT transfer

		#[pallet::weight(10_000)]
		pub fn fundProp(
			powner: OriginFor<T>,
			account: T::AccountId,
			index: PropIndex,
			index2:FundIndex,
						
		)-> DispatchResultWithPostInfo{
			let mut fund = Self::funds(index2).ok_or(Error::<T>::InvalidIndex)?;
			let mut pro =Props::<T>::get();
			let price =pro.price;
			
			//Pay the proposal owner From Treasurery
			T::Currency::transfer(
				&Self::fund_account_id(index),
				&account,
				price,
				ExistenceRequirement::AllowDeath,
			)?;	
		
			//Kazu:Determine which contributor is included into the proposal
			//Kazu:nbr of participants

			//let mut part=4;
			//while part !=0{
			//
			//}
		
			//Kazu:calculate purcentage based on number of contributors
			//let percentage = 1/part;
		
			//distribute NFTs to contributors
			//let transf = rml_nft::Pallet::<T>::tranfer(account,contrib,token,percentage);
		Ok(().into())
		}

		/// Withdraw full balance of a contributor to a fund
		#[pallet::weight(10_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			#[pallet::compact] index: FundIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;
			let now = <frame_system::Pallet<T>>::block_number();
			// ensure!(fund.end < now, Error::<T>::FundStillActive);

			let balance = Self::contribution_get(index, &who);
			ensure!(balance > Zero::zero(), Error::<T>::NoContribution);

			// Return funds to caller without charging a transfer fee
			let _ = T::Currency::resolve_into_existing(
				&who,
				T::Currency::withdraw(
					&Self::fund_account_id(index),
					balance,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Update storage
			Self::contribution_kill(index, &who);
			fund.raised = fund.raised.saturating_sub(balance);
			<Funds<T>>::insert(index, &fund);

			Self::deposit_event(Event::Withdrew(who, index, balance, now));

			Ok(().into())
		}

		/// Dissolve an entire crowdfund after its retirement period has expired.
		/// Anyone can call this function, and they are incentivized to do so because
		/// they inherit the deposit.
		#[pallet::weight(10_000)]
		pub fn dissolve(origin: OriginFor<T>, index: FundIndex) -> DispatchResultWithPostInfo {
			let reporter = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::Pallet<T>>::block_number();
			// ensure!(now >= fund.end + T::RetirementPeriod::get(), Error::<T>::FundNotRetired);

			let account = Self::fund_account_id(index);

			// Dissolver collects the deposit and any remaining funds
			let _ = T::Currency::resolve_creating(
				&reporter,
				T::Currency::withdraw(
					&account,
					fund.deposit + fund.raised,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dissolved(index, now, reporter));

			Ok(().into())
		}

		/// Dispense a payment to the beneficiary of a successful crowdfund.
		/// The beneficiary receives the contributed funds and the caller receives
		/// the deposit as a reward to incentivize clearing settled crowdfunds out of storage.
		#[pallet::weight(10_000)]
		pub fn dispense(origin: OriginFor<T>, index: FundIndex) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::Pallet<T>>::block_number();

			// ensure!(now >= fund.end, Error::<T>::FundStillActive);

			// // Check that the fund was actually successful
			// ensure!(fund.raised >= fund.goal, Error::<T>::UnsuccessfulFund);

			let account = Self::fund_account_id(index);

			// Beneficiary collects the contributed funds
			let _ = T::Currency::resolve_creating(
				&fund.beneficiary,
				T::Currency::withdraw(
					&account,
					fund.raised,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Caller collects the deposit
			let _ = T::Currency::resolve_creating(
				&caller,
				T::Currency::withdraw(
					&account,
					fund.deposit,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dispensed(index, now, caller));
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
		/// The account ID of the fund pot.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache the
		/// value and only call this once.
		pub fn fund_account_id(index: FundIndex) -> T::AccountId {
			PALLET_ID.into_sub_account(index)
		}

		/// Find the ID associated with the fund
		///
		/// Each fund stores information about its contributors and their contributions in a child trie
		/// This helper function calculates the id of the associated child trie.
		pub fn id_from_index(index: FundIndex) -> child::ChildInfo {
			let mut buf = Vec::new();
			buf.extend_from_slice(b"crowdfnd");
			buf.extend_from_slice(&index.to_le_bytes()[..]);

			child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
		}

		/// Record a contribution in the associated child trie.
		pub fn contribution_put(index: FundIndex, who: &T::AccountId, balance: &BalanceOf<T>) {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::put(&id, b, &balance));
		}

		/// Lookup a contribution in the associated child trie.
		pub fn contribution_get(index: FundIndex, who: &T::AccountId) -> BalanceOf<T> {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
		}

		/// Remove a contribution from an associated child trie.
		pub fn contribution_kill(index: FundIndex, who: &T::AccountId) {
			let id = Self::id_from_index(index);
			who.using_encoded(|b| child::kill(&id, b));
		}

		/// Remove the entire record of contributions in the associated child trie in a single
		/// storage write.
		pub fn crowdfund_kill(index: FundIndex) {
			let id = Self::id_from_index(index);
			// The None here means we aren't setting a limit to how many keys to delete.
			// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
			// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
			child::kill_storage(&id, None);
		}
	}
}
