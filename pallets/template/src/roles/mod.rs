//! # Roles
//!
//! Definition and implementation of the different Roles found in FairSquares

mod items;
pub use super::*;
pub use crate::roles::items::*;
pub type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type BalanceOf2<T> = <<T as pallet_democracy::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type Contributors<T> = Vec<AccountIdOf<T>>;
pub type HouseIndex = u32;
pub type ProposalIndex = u32;
pub type ContributionIndex = u32;
pub type Bool = bool;
pub type NftOf<T> = Vec<T>;
pub type ClassOf<T> = <T as pallet_nft::Config>::NftClassId;
pub type InstanceOf<T> = <T as pallet_nft::Config>::NftInstanceId;
pub type NfT<T> = NftL::TokenByOwnerData<T>;
pub type Idle<T> = (Vec<HouseSeller<T>>,Vec<Servicer<T>>);


pub const HOUSE_CLASS:u32=1000;
pub const APT_CLASS:u32=2000;
pub const COMPUTER_CLASS:u32=3000;


//-------------------------------------------------------------------------------------
//-------------INVESTOR STRUCT DECLARATION & IMPLEMENTATION_BEGIN----------------------
#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Investor<T:Config> {
    pub account_id:T::AccountId,
    pub nft_index:NftOf<u32>,
    pub age:BlockNumberOf<T>,
    pub share:BalanceOf<T>,
    pub selections:u32,
}

impl<T:Config> Investor<T> where roles::Investor<T>: EncodeLike<roles::Investor<T>>{

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

    //-------------------------------------------------------------------
    //-------------INVESTOR CONTRIBUTION METHOD_BEGIN--------------------
    pub fn contribute(mut self, origin:OriginFor<T>,value:BalanceOf<T>) -> DispatchResult{
        
        let who = ensure_signed(origin)?;
	ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
	
	let now = <frame_system::Pallet<T>>::block_number();    
    let wperc = Pallet::<T>::u32_to_balance_option(100000);
    
    let idx = ContribIndex::<T>::get()+1;
    ContribIndex::<T>::put(&idx);
    <T as pallet::Config>::Currency::transfer(
        &who,
        &TREASURE_PALLET_ID.into_account(),
        value,
        ExistenceRequirement::AllowDeath,
    )?;
    let total_fund:BalanceOf<T> = Pallet::<T>::pot();
    
    let share = wperc.unwrap()*value/total_fund;
    self.share = share.clone();
	let c1=Contribution::<T>::new(value.clone(),idx);
    let inv = Some(self.clone());

    InvestorLog::<T>::mutate(&self.account_id,|val|{
        *val = inv;
    });
        if ContributionsLog::<T>::contains_key(&self.account_id){
            ContributionsLog::<T>::mutate(&self.account_id, |val|{
                
                let rec = val.clone().unwrap();
                let b = rec.1 + c1.amount;
                *val = Some((now,b,c1));
            })
        } else {
            let id = self.account_id;
            let v0 = c1;
            ContributionsLog::<T>::insert(id,(now,value,v0));
        }
        

        

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
pub struct HouseSeller<T: Config>{
    pub account_id:T::AccountId,
    pub nft_index:NftOf<u32>,
    pub age:BlockNumberOf<T>,
}
impl<T:Config> HouseSeller<T> where roles::HouseSeller<T>: EncodeLike<roles::HouseSeller<T>>{

    //--------------------------------------------------------------------
    //-------------HOUSE OWNER CREATION METHOD_BEGIN----------------------
    pub fn new(acc: OriginFor<T>) -> DispatchResult{
        let caller = ensure_signed(acc).unwrap();
        let now = <frame_system::Pallet<T>>::block_number(); 
        //ensure!(HouseSellerLog::<T>::contains_key(&caller)==false,Error::<T>::NoneValue);      
        
            let hw = HouseSeller{
                account_id: caller.clone(),
                nft_index: Vec::new(),
                age: now.clone(),		
            };
            WaitingList::<T>::mutate(|val|{
                val.0.push(hw);
            });         
            Ok(().into())

        } 

    //-------------HOUSE OWNER CREATION METHOD_END----------------------
    //------------------------------------------------------------------
        
    //-----------------------------------------------------------------
    //-------------MINT HOUSE METHOD_BEGIN-----------------------------
    
    pub fn mint_house(&self,origin:OriginFor<T>){
        let _creator = ensure_signed(origin);
        let now = <frame_system::Pallet<T>>::block_number();
        let idx = HouseInd::<T>::get()+1;
        HouseInd::<T>::put(idx.clone());
        let house = House{
            owners: vec![self.account_id.clone()],
            nft_index: idx,
            age: now,
            index: idx.clone(),
        };
        MintedHouseLog::<T>::insert(idx,house);

    }
    //-------------MINT HOUSE METHOD_END-------------------------------
    //-----------------------------------------------------------------

    //-----------------------------------------------------------------
    //-------------PROPOSAL CREATION METHOD_BEGIN----------------------

    pub fn new_proposal(self,origin: OriginFor<T>,value: BalanceOf<T>,hindex:u32,metadata:Vec<u8>) -> DispatchResult{
        let creator = ensure_signed(origin.clone())?;
        let now = <frame_system::Pallet<T>>::block_number();
        
        //Make sure that the Seller has enough funds to make a deposit
        //let balance  = <T as pallet::Config>::Currency::free_balance(&creator);
        //let balance1 = Pallet::<T>::balance_to_u32_option(balance).unwrap();
        //let balance2 = Pallet::<T>::u32_to_balance_option2(balance1).unwrap();
        let deposit0= <T as DMC::Config>::MinimumDeposit::get();
        //ensure!(balance2>deposit0,Error::<T>::NotEnoughFunds);

        let pindex = ProposalInd::<T>::get()+1;
        ProposalInd::<T>::put(pindex.clone());

        if MintedHouseLog::<T>::contains_key(hindex.clone()){
        if ProposalLog::<T>::contains_key(pindex.clone())==false{

            
            //add proposal to DMC voting queue
            let house = MintedHouseLog::<T>::get(hindex);
            let proposal_hash = T::Hashing::hash(&metadata[..]);
            DMC::Pallet::<T>::propose(origin.clone(),proposal_hash.clone(),deposit0).ok();

            DMC::Pallet::<T>::note_imminent_preimage(origin.clone(),metadata.clone()).ok();
            
            //Start Referendum with a 'SimpleMajority' threshold
            let threshold = DMC::VoteThreshold::SimpleMajority;
            let delay = <T as Config>::Delay::get();
            DMC::Pallet::<T>::internal_start_referendum(proposal_hash,threshold,delay);

            
            //Select Investors for nft ownership
            //-------------------------------------------------------------------------------
            //mint a nft with the same index as HouseInd here                       
            //mint
            
            let data:BoundedVecOfUnq<T> = metadata.try_into().unwrap();
            let cl_id:ClassOf<T> = HOUSE_CLASS.into();
            let inst_id:InstanceOf<T> = hindex.into();

            let cls = NftL::Pallet::<T>::do_create_class(
                creator.clone(),
                cl_id.clone(),
                Default::default(),
                data.clone()
            )?;            
            let _nft = NftL::Pallet::<T>::do_mint(
                creator.clone(),
                cls.0,
                inst_id.clone(),
                data
            );
            let hi:InstanceOf<T> = hindex.clone().into();

            let own = NftL::TokenByOwner::<T>::get(creator.clone(),(cls.0,hi)).unwrap();
            if !(MintedNftLog::<T>::contains_key(&creator,&hindex)){
                MintedNftLog::<T>::insert(creator,hindex,(cl_id,inst_id,own));
            }         
            //----------------------------------------------------------------------------------
            ReserveFunds::<T>::mutate(|val|{
                *val += value.clone();
            });
            let store = (now,value,house,false);
            
            ProposalLog::<T>::insert(pindex,store);
            
            }
        }

        Ok(().into())
    }

    
    //-------------PROPOSAL CREATION METHOD_END----------------------
    //-----------------------------------------------------------------
    
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
