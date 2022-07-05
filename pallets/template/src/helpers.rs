pub use super::*;

impl<T: Config> Pallet<T> {

    pub fn check_storage(caller: T::AccountId) -> DispatchResult {
		ensure!(HouseSellerLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(InvestorLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(ServicerLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		ensure!(TenantLog::<T>::contains_key(&caller) == false, Error::<T>::OneRoleAllowed);
		Ok(().into())
	}

    pub fn u32_to_balance_option(input: u32) -> Option<BalanceOf<T>> {
        input.try_into().ok()
      }

      pub fn u32_to_balance_option2(input: u32) -> Option<BalanceOf2<T>> {
        input.try_into().ok()
      }
  
     pub fn balance_to_u32_option(input: BalanceOf<T>) -> Option<u32> {
        input.try_into().ok()
      }
     
      pub fn balance_to_u32_option2(input: BalanceOf2<T>) -> Option<u32> {
        input.try_into().ok()
      }
     
      pub fn approve_account(who: T::AccountId) {
        let waitlist = WaitingList::<T>::get();
        let sellers =  waitlist.0;
        let servicers = waitlist.1;
        for sell in sellers.iter(){
           if sell.account_id == who.clone(){
              HouseSellerLog::<T>::insert(&who,sell.clone());
              let index = sellers.iter().position(|x| *x == *sell).unwrap();
              WaitingList::<T>::mutate(|val|{
                 val.0.remove(index);
              })
           }
        }
        for serv in servicers.iter(){
           if serv.account_id == who.clone(){
              ServicerLog::<T>::insert(&who,serv);
              let index = servicers.iter().position(|x| *x == *serv).unwrap();
              WaitingList::<T>::mutate(|val|{
                 val.0.remove(index);
              })
           }
        }
      }

      pub fn mint_house_nft(creator: T::AccountId, hindex:u32,metadata:Vec<u8>){
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
          ).unwrap();            
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
        }

     pub fn pot() -> BalanceOf<T> {
          <T as pallet::Config>::Currency::free_balance(&TREASURE_PALLET_ID.into_account())
          // Must never be less than 0 but better be safe.
          .saturating_sub(<T as pallet::Config>::Currency::minimum_balance())
        }

    //During Investors vote, Houses linked to an approved proposal are removed from
      //the MintedHouse storage, and the boolean in the corresponding Proposal_storage
      //is turned to true.
      ///Fractional_transfer takes care of nft ownership & share distribution, as well as
      ///update of related storages.
      pub fn fractional_transfer(from:T::AccountId, to:Vec<(T::AccountId,BalanceOf<T>)>,p_index:ProposalIndex)-> DispatchResult{
        //Check that Proposal has been accepted
        let mut proposal = ProposalLog::<T>::get(p_index.clone());
        ensure!(proposal.clone().3==true,Error::<T>::UnsuccessfulFund);

        let house =  proposal.clone().2;
        let house_index = house.clone().index;
        //Check that sending account is a seller
        ensure!(HouseSellerLog::<T>::contains_key(&from),Error::<T>::NotSellerAccount);
        
        //Check that this seller has ownership of this house 
        let howner = house
                       .clone()
                       .owners;
        ensure!(howner.contains(&from), Error::<T>::NoAccount);

        //remove Seller from house owners list
        proposal.2.owners.remove(0);

        //Get nft data from minted nft storage
        let _nft_instance = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().2.instance;
        let class_id:ClassOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().0;
        let instance_id:InstanceOf<T> = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap().1;
        let mut nft_item = MintedNftLog::<T>::get(&from,house_index.clone()).unwrap();
        MintedNftLog::<T>::remove(&from,house_index.clone());
        
        //Remove nft_index from house_seller struct
        let mut seller0 = (HouseSellerLog::<T>::get(&from)).unwrap();
        seller0.nft_index.remove(0);
        let seller = Some(seller0);
        HouseSellerLog::<T>::mutate(&from,|val|{
           *val = seller;
        });

        //Nft share redistribution is done in the function do_transfer of the nft_pallet
        //Get the house value from the storage
        let value = Self::balance_to_u32_option(proposal.1).unwrap();

        for i in to{
           //Calculate nft share from amount contributed to the house
           let contribution = Self::balance_to_u32_option(i.1).unwrap();
           let share = (contribution*100000)/&value;
           
           //Update minted nft log with new owners
           
           if !(MintedNftLog::<T>::contains_key(i.0.clone(),house_index.clone())){
              nft_item.2.percent_owned = share.clone();
              MintedNftLog::<T>::insert(&i.0,&house_index,nft_item.clone());
           }
           //
           //Redistribute nft share
           NftL::Pallet::<T>::do_transfer(class_id.clone(),instance_id.clone(),from.clone(),i.clone().0,share).ok();
           
          
           //Update the list of owners in the house structs found in ProposalLog_storage & remove house item from minted house
           proposal.2.owners.push(i.0);       
        
        }
        ProposalLog::<T>::mutate(&p_index,|val|{
           *val = proposal;
        });


        Ok(().into())


     }
}