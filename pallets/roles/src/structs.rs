//! # structs
//!
//! Definition and implementation of the different structs found in FairSquares


pub use super::*;
pub use frame_support::{
    dispatch::{DispatchResult,EncodeLike},
    pallet_prelude::*,
    inherent::Vec,
    sp_runtime::traits::{AccountIdConversion,Hash, Zero,Saturating},
    storage::{child},
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency,LockableCurrency, WithdrawReasons},
    PalletId,
    assert_ok,		
 };
pub use frame_system::{pallet_prelude::*,ensure_signed};
pub use frame_support::pallet_prelude::*;
pub use scale_info::{prelude::vec,TypeInfo};

pub type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Idle<T> = (Vec<HouseSeller<T>>,Vec<Servicer<T>>);

//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T:Config> {
    pub account_id:T::AccountId,
    pub nft_index:Vec<u32>,
    pub age:BlockNumberOf<T>,
    pub share:BalanceOf<T>,
    pub selections:u32,
}

impl<T:Config> Investor<T> where structs::Investor<T>: EncodeLike<structs::Investor<T>>{

    //-------------------------------------------------------------------
    //-------------NEW INVESTOR CREATION METHOD_BEGIN--------------------
    pub fn new(acc:OriginFor<T>) -> Self{
        let caller = ensure_signed(acc).unwrap();
        let now = <frame_system::Pallet<T>>::block_number();
            
            let inv = Investor{
                account_id: caller.clone(),
                nft_index: Vec::new(),
                age: now,
                share:Zero::zero(),
                selections:0,
            };
            
            InvestorLog::<T>::insert(caller.clone(),inv);
        
        
        Investor{
            account_id: caller,
            nft_index: Vec::new(),
            age: now,
            share:Zero::zero(),
            selections:0,
        }

        }
    //-------------NEW INVESTOR CREATION METHOD_END--------------------
    //-----------------------------------------------------------------



}
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-----------------------------------------------------------------------------------





//--------------------------------------------------------------------------------------
//-------------HOUSE OWNER STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseSeller<T: Config>{
    pub account_id:T::AccountId,
    pub nft_index:Vec<u32>,
    pub age:BlockNumberOf<T>,
}
impl<T:Config> HouseSeller<T> where structs::HouseSeller<T>: EncodeLike<structs::HouseSeller<T>>{

    //--------------------------------------------------------------------
    //-------------HOUSE OWNER CREATION METHOD_BEGIN----------------------
    pub fn new(acc: OriginFor<T>) -> DispatchResult{
        let caller = ensure_signed(acc).unwrap();
        let now = <frame_system::Pallet<T>>::block_number(); 
        ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);      
        
            let hw = HouseSeller{
                account_id: caller,
                nft_index: Vec::new(),
                age: now,		
            };
            
            WaitingList::<T>::mutate(|val|{
                val.0.push(hw);
            });         
            
            Ok(().into())

        } 

    //-------------HOUSE OWNER CREATION METHOD_END----------------------
    //------------------------------------------------------------------
 
    
}
//-------------HOUSE OWNER STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//--------------------------------------------------------------------------------------




//--------------------------------------------------------------------------------------
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Tenant<T:Config>{
    pub account_id: T::AccountId,
    pub rent:BalanceOf<T>,
    pub age:BlockNumberOf<T>,
}
impl<T:Config> Tenant<T> {    
    pub fn new(acc:OriginFor<T>)-> Self{
        let caller = ensure_signed(acc).unwrap();        
        let now = <frame_system::Pallet<T>>::block_number();
        Tenant{
            account_id: caller,
            rent: Zero::zero(),
            age:now,
        }
        
    }
}
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------


//--------------------------------------------------------------------------------------
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Servicer<T:Config>{
    pub account_id: T::AccountId,
    pub age:BlockNumberOf<T>,
}
impl<T:Config> Servicer<T> {    
    pub fn new(acc:OriginFor<T>) -> DispatchResult{
        let caller = ensure_signed(acc).unwrap();        
        let now = <frame_system::Pallet<T>>::block_number();
        let sv = Servicer{
            account_id: caller,
            age:now,
        };
        WaitingList::<T>::mutate(|val|{
            val.1.push(sv);
        });
        Ok(().into())
        
    }
}
//-------------Servicer STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------