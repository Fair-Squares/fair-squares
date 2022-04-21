
mod items;
pub use super::*;
pub use crate::roles::items::*;
pub type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Contributors<T> = Vec<AccountIdOf<T>>;
pub type HouseIndex = u32;
pub type ProposalIndex = u32;
pub type ContributionIndex = u32;
pub type Bool = bool;
pub type NftOf<T> = Vec<T>;


#[derive(Clone, Encode, Decode,PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct House<T:Config> {
    pub owners:Vec<T::AccountId>,
    pub nft:u32,
    pub age:BlockNumberOf<T>,
}
impl<T:Config> Default for House<T>{
    fn default() -> Self{
        Self{owners: Vec::new(),nft:0,age:<frame_system::Pallet<T>>::block_number()}
    }
}

//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T:Config,U> {
    pub account_id:T::AccountId,
    pub nft:NftOf<U>,
    pub age:BlockNumberOf<T>,
}

impl<T:Config,U> Investor<T,U> where roles::Investor<T, U>: EncodeLike<roles::Investor<T, u32>>{

    //-------------------------------------------------------------------
    //-------------NEW INVESTOR CREATION METHOD_BEGIN--------------------
    pub fn new(acc:T::AccountId,_nft:U){
        let now = <frame_system::Pallet<T>>::block_number();
        
        if InvestorLog::<T>::contains_key(&acc)==false{
            let inv = Investor{
                account_id: acc,
                nft: Vec::new(),
                age: now,		
            };
            InvestorLog::<T>::insert(inv.account_id.clone(),vec![inv]);
        } else {
            let _message = "Role already attributed";
                //return the above string in an event          

        }          

        }
    //-------------NEW INVESTOR CREATION METHOD_END--------------------
    //-----------------------------------------------------------------

    //-------------------------------------------------------------------
    //-------------INVESTOR CONTRIBUTION METHOD_BEGIN--------------------
    pub fn contribute(self, origin:OriginFor<T>,value:BalanceOf<T>) -> DispatchResult{
        
        let who = ensure_signed(origin)?;
	ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
	
	let now = <frame_system::Pallet<T>>::block_number();
    let idx = ContribIndex::<T>::get()+1;
	let c1=Contribution::new(&self.account_id,&value);
        if ContributionsLog::<T>::contains_key(&idx){
            ContributionsLog::<T>::mutate(&idx, |val|{
                val.1 += *c1.amount;
            })
        } else {
            let id = idx;
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
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct HouseOwner<T: Config,U>{
    pub account_id:T::AccountId,
    pub nft:NftOf<U>,
    pub age:BlockNumberOf<T>,
}
impl<T:Config,U> HouseOwner<T,U> where roles::HouseOwner<T, U>: EncodeLike<roles::HouseOwner<T, u32>>{

    //--------------------------------------------------------------------
    //-------------HOUSE OWNER CREATION METHOD_BEGIN----------------------
    pub fn new(acc:T::AccountId,_nft:U){

        let now = <frame_system::Pallet<T>>::block_number();        
        if HouseOwnerLog::<T>::contains_key(&acc)==false{
            let hw = HouseOwner{
                account_id: acc,
                nft: Vec::new(),
                age: now,		
            };
            HouseOwnerLog::<T>::insert(hw.account_id.clone(),vec![hw]);
        } else {
            let _message = "Role already attributed";
                //return the above string in an event         

        }       

        } 

    //-------------HOUSE OWNER CREATION METHOD_END----------------------
    //------------------------------------------------------------------
        
    //-----------------------------------------------------------------
    //-------------MINT HOUSE METHOD_BEGIN-----------------------------

    pub fn mint_house(&self,origin:OriginFor<T>){
        let creator = ensure_signed(origin);
        let now = <frame_system::Pallet<T>>::block_number();
        let idx = HouseInd::<T>::get()+1;
        HouseInd::<T>::put(idx.clone());
        let house = House{
            owners: vec![self.account_id.clone()],
            nft: idx,
            age: now,
        };
        MintedHouseLog::<T>::insert(idx,house);
    }
    //-------------MINT HOUSE METHOD_END-------------------------------
    //-----------------------------------------------------------------

    //-----------------------------------------------------------------
    //-------------PROPOSAL CREATION METHOD_BEGIN----------------------

    pub fn new_proposal(self,origin: OriginFor<T>,value: BalanceOf<T>,hindex:u32) -> DispatchResult{
        let creator = ensure_signed(origin)?;
        let now = <frame_system::Pallet<T>>::block_number();
        let deposit = <T as pallet::Config>::SubmissionDeposit::get();
        let imb = <T as pallet::Config>::Currency::withdraw(
            &creator,
            deposit,
            WithdrawReasons::TRANSFER,
            ExistenceRequirement::AllowDeath,
        )?;
        let pindex = ProposalInd::<T>::get()+1;
        ProposalInd::<T>::put(pindex.clone());

        if MintedHouseLog::<T>::contains_key(hindex.clone()){
        if ProposalLog::<T>::contains_key(pindex.clone())==false{
            let mut v = Vec::new();
            <T as pallet::Config>::Currency::resolve_creating(&self.account_id, imb);
            v.push(self.account_id);
            let house = MintedHouseLog::<T>::get(hindex);
            //mint a nft with the same index as HouseInd here
            let store = (now,value,house,false);
            ProposalLog::<T>::insert(pindex,store);
            }
        }

        Ok(().into())
    }

    pub fn destroy_proposal(){}
    //-------------PROPOSAL CREATION METHOD_END----------------------
    //-----------------------------------------------------------------
    
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

