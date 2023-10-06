//! # structs
//!
//! Definition and implementation of the different structs found in FairSquares

pub use super::*;
pub use frame_support::{
	assert_ok,
	dispatch::{DispatchResult},
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
	storage::{child,bounded_vec::BoundedVec},
	traits::{
		UnfilteredDispatchable,Currency, ExistenceRequirement, Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	dispatch::GetDispatchInfo,
	PalletId,
};
pub use sp_std::vec::Vec;
pub use frame_system::{ensure_signed, ensure_root, pallet_prelude::*, RawOrigin};
pub use scale_info::{prelude::vec, TypeInfo};
pub use serde::{Deserialize, Serialize};

pub type BalanceOf<T> =
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = BlockNumberFor<T>;
///This enum contains the roles selectable at account creation
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Accounts {
	INVESTOR,
	#[default]
	SELLER,
	TENANT,
	SERVICER,
	NOTARY,
	REPRESENTATIVE,
}

#[derive(Clone,Copy, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Proposal<T: Config>{
	pub account_id: T::AccountId,
	pub role: Option<Accounts>,
	pub block: BlockNumberOf<T>,
	pub proposal_hash: T::Hash,
	pub proposal_index: u32,
	pub session_closed: bool, 
	pub approved: bool,
}
impl<T: Config> Proposal<T>{
	pub fn new(acc:T::AccountId, role: Option<Accounts>,proposal: T::Hash) -> Self{
		let now = <frame_system::Pallet<T>>::block_number();
		let proposal_hash =  T::Hashing::hash_of(&proposal);
		let proposal = Proposal {account_id: acc,role,block: now,proposal_hash,proposal_index:0,session_closed:false,approved:false};
		proposal
	}
}

//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone,Copy, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub share: BalanceOf<T>,
	pub selections: u32,
}

impl<T: Config> Investor<T>
/*where
	types::Investor<T>: EncodeLike<types::Investor<T>>,*/
{
	//-------------------------------------------------------------------
	//-------------NEW INVESTOR CREATION METHOD_BEGIN--------------------
	pub fn new(acc: T::AccountId,) -> Self {
		let now = <frame_system::Pallet<T>>::block_number();

		let inv =
			Investor { account_id: acc.clone(), age: now, share: Zero::zero(), selections: 0 };

		InvestorLog::<T>::insert(acc, &inv);
        inv

	}
	//-------------NEW INVESTOR CREATION METHOD_END--------------------
	//-----------------------------------------------------------------
}
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-----------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------HOUSE SELLER STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone,Copy,Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseSeller<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
}
impl<T: Config> HouseSeller<T>
/*where
	types::HouseSeller<T>: EncodeLike<types::HouseSeller<T>>,*/
{
	//--------------------------------------------------------------------
	//-------------HOUSE SELLER CREATION METHOD_BEGIN----------------------
	pub fn new(acc: T::AccountId) -> Self {
		let now = <frame_system::Pallet<T>>::block_number();
		let hw = HouseSeller { 
            account_id: acc.clone(), 
            age: now, activated: false
        };

		SellerApprovalList::<T>::mutate(|list| {
			list.push(hw.clone());
		});

        hw
	}

	//-------------HOUSE SELLER CREATION METHOD_END----------------------
	//------------------------------------------------------------------
}
//-------------HOUSE SELLER STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//--------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Copy,Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Tenant<T: Config> {
	pub account_id: T::AccountId,
	pub rent: BalanceOf<T>,
	pub age: BlockNumberOf<T>,
	pub asset_account: Option<T::AccountId>,
	pub contract_start: BlockNumberOf<T>,
	pub remaining_rent: BalanceOf<T>,
	pub remaining_payments: u8,
	pub registered: bool,
}
impl<T: Config> Tenant<T> {
	pub fn new(acc: T::AccountId) -> Self {
		let now = <frame_system::Pallet<T>>::block_number();
		let tenant = Tenant {
			account_id: acc.clone(),
			rent: Zero::zero(),
			age: now,
			asset_account: None,
			contract_start: now,
			remaining_rent: Zero::zero(),
			remaining_payments: 0,
			registered: false,
		};
		TenantLog::<T>::insert(acc, &tenant);
        tenant
	}
}
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Copy,Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Servicer<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
}
impl<T: Config> Servicer<T> {
	pub fn new(acc: T::AccountId) -> Self {
		let now = <frame_system::Pallet<T>>::block_number();
		let sv =
			Servicer { account_id: acc.clone(), age: now, activated: false};

		ServicerApprovalList::<T>::mutate(|list| {
			list.push(sv.clone());
		});
		sv

	}
}
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------
//-------------REPRESENTATIVE STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------

#[derive(Clone,Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Representative<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
	pub assets_accounts: Vec<T::AccountId>,
	pub index: u32,
}
impl<T: Config> Representative<T>
/*where
	types::Representative<T>: EncodeLike<types::Representative<T>>,*/
{
	//--------------------------------------------------------------------
	//-------------REPRESENTATIVE CREATION METHOD_BEGIN----------------------
	pub fn new(acc: T::AccountId) -> Self {
		let now = <frame_system::Pallet<T>>::block_number();

		if !RepresentativeLog::<T>::contains_key(acc.clone()) {
			let rep = Representative::<T> {
				account_id: acc.clone(),
				age: now,
				activated: false,
				assets_accounts: Vec::new(),
				index: Default::default(),
			};
			RepApprovalList::<T>::insert(acc.clone(), rep.clone());
            rep
		} else {
			let rep = RepresentativeLog::<T>::get(acc.clone()).unwrap();
			RepApprovalList::<T>::insert(acc, rep.clone());
            rep
		}

	}

	//-------------HOUSE REPRESENTATIVE CREATION METHOD_END----------------------
	//------------------------------------------------------------------
}

//-------------REPRESENTATIVE STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------
//-------------NOTARY STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone,Copy,Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Notary<T: Config> {
	pub account_id: T::AccountId,
	pub age: BlockNumberOf<T>,
	pub activated: bool,
}
impl<T: Config> Notary<T>
/*where
	types::Notary<T>: EncodeLike<types::Notary<T>>,*/
{
	pub fn new(acc: OriginFor<T>) -> Self {
		let caller = ensure_signed(acc).unwrap();
		let now = <frame_system::Pallet<T>>::block_number();
		let notary =
			Notary { account_id: caller.clone(), age: now, activated: false};
		NotaryApprovalList::<T>::mutate(|list| {
			list.push(notary.clone());
		});

        notary

	}
}

//-------------NOTARY STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-------------------------------------------------------------------------------------