
mod items;
pub use super::*;
pub use crate::roles::items::*;
pub type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Contributors<T> = Vec<AccountIdOf<T>>;
pub type HouseIndex = u32;
pub type ContributionIndex = u32;
pub type Bool = bool;
pub const INVESTOR_ROLE: u8 = 1;
pub const HOUSE_OWNER_ROLE: u8 = 2;
pub const TENANT_ROLE: u8 = 3;


//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T:Config,U> {
    pub account_id:T::AccountId,
    pub nft:Vec<U>,
    pub age:BlockNumberOf<T>,
}

impl<T:Config,U> Investor<T,U> where roles::Investor<T, U>: EncodeLike<roles::Investor<T, u32>>{

    //-------------------------------------------------------------------
    //-------------NEW INVESTOR CREATION METHOD_BEGIN--------------------
    pub fn new(acc:T::AccountId,_nft:U)-> Self{
        let now = <frame_system::Pallet<T>>::block_number();
        if UsersLog::<T>::contains_key(&acc)==false{
            UsersLog::<T>::insert(&acc,vec![INVESTOR_ROLE]);
        } else {
            let a = UsersLog::<T>::get(&acc);
            if a.iter().any(|&i| i==INVESTOR_ROLE){
                let _message = "Role already attributed";
                //return the above string in an event
            } else{
                UsersLog::<T>::mutate(&acc,|val|{
                    val.push(INVESTOR_ROLE);
                })
            }           

        }         
            Investor{
                account_id: acc,
                nft: Vec::new(),
                age: now,		
            }        
        }
    //-------------NEW INVESTOR CREATION METHOD_END--------------------
    //-----------------------------------------------------------------

    //-------------------------------------------------------------------
    //-------------INVESTOR CONTRIBUTION METHOD_BEGIN--------------------
    pub fn contribute(self, origin:OriginFor<T>,value:BalanceOf<T>) -> DispatchResultWithPostInfo{
        
        let who = ensure_signed(origin)?;
	ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
	
	let now = <frame_system::Pallet<T>>::block_number();
	let c1=Contribution::new(&self.account_id,&value);
        if ContributionsLog::<T>::contains_key(c1.account){
            ContributionsLog::<T>::mutate(c1.account, |val|{
                val.1 += *c1.amount;
            })
        } else {
            let id = (&self.account_id).clone();
            let v0 = vec![self];
            ContributionsLog::<T>::insert(id,(now,value,v0));
        }
        

        <T as pallet::Config>::Currency::transfer(
            &who,
            &TREASURE_PALLET_ID.into_account(),
            value,
            ExistenceRequirement::AllowDeath,
        )?;

        Ok(().into())
    }
    //-------------INVESTOR CONTRIBUTION METHOD_END--------------------
    //-----------------------------------------------------------------
}
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//-----------------------------------------------------------------------------------





//--------------------------------------------------------------------------------------
//-------------HOUSE OWNER STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseOwner<T: Config,U>{
    pub account_id:T,
    pub nft:U,
    pub age:BlockNumberOf<T>,
}
//-------------HOUSE OWNER STRUCT DECLARATION & IMPLEMENTATION_END----------------------
//--------------------------------------------------------------------------------------




//--------------------------------------------------------------------------------------
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_BEGIN---------------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Tenant<T:Config,U>{
    pub account_id: T::AccountId,
    pub rent:U,
    pub age:BlockNumberOf<T>,
}
impl<T:Config,U> Tenant<T,U>{
    
    pub fn new(acc:T::AccountId,rent:U)-> Self{
        let now = <frame_system::Pallet<T>>::block_number();
        Tenant{
            account_id: acc,
            rent: rent,
            age:now,
        }
        
    }
}
//-------------TENANT STRUCT DECLARATION & IMPLEMENTATION_END---------------------------
//--------------------------------------------------------------------------------------
