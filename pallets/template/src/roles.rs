
pub use super::*;
pub use crate::items::*;

pub use frame_support::{
   dispatch::DispatchResult,
   pallet_prelude::*,
   codec::{Encode, Decode},
   traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons}
};
pub use frame_system::{pallet_prelude::*,ensure_signed};
use frame_support::inherent::Vec;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Investor<T: Config> {
    pub account_id: AccountIdOf<T>,
}
impl<T: Config> Investor<T>{
    pub fn new(account_id: AccountIdOf<T>)-> Self{
        Self {
            account_id
        }        
    }

    pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !Investors::<T>::contains_key(&account_id) {
         let investor = Investor { account_id: account_id.clone() };
         Investors::<T>::insert(account_id.clone(), investor);
         return true
      }
      false
    }

    pub fn add_contribution_fund(&self, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
      
      let _account = self.account_id.clone();
      let block_number = <frame_system::Pallet<T>>::block_number();
      let contribution = Contribution {
         amount: amount,
         timestamp: block_number
      };

      let mut total_fund = amount.clone();
      let wrap_fund = <FundAmount<T>>::get();
      if !wrap_fund.is_none() {
         total_fund += wrap_fund.unwrap();
      }

      if !Contributions::<T>::contains_key(&_account) {

         Contributions::<T>::insert(&_account, (amount, 0, self.account_id.clone()));

         let mut contribution_list = Vec::new();
         contribution_list.push(contribution);
         ContributionLog::<T>::insert(&_account, contribution_list);
      } else {
         ContributionLog::<T>::mutate(&_account, |val| {
            val.push(contribution);
         });

         Contributions::<T>::mutate(&_account, |val| {
             //val.0 += amount;
             let unwrap_val = val.clone().unwrap();
             let contrib = (unwrap_val.0 + amount, unwrap_val.1, unwrap_val.2);
             *val = Some(contrib);
         });
      }

      <FundAmount<T>>::put(total_fund.clone());

      let contributions_iter = Contributions::<T>::iter();

      // Update the share for all contributors
      for item in contributions_iter {
         
         let wrap_percent = self.u64_to_balance_option(100000);
         let share = wrap_percent.unwrap() * item.1.0.clone() / total_fund;

         Contributions::<T>::mutate(item.0, |val| {

            let unwrap_val = val.clone().unwrap();
            let contrib = (unwrap_val.0, self.balance_to_u32_option(share).unwrap(), unwrap_val.2);
            *val = Some(contrib);
        });
      }

      Ok(().into())
    }

    fn u64_to_balance_option(&self, input: u64) -> Option<BalanceOf<T>> {
      input.try_into().ok()
    }

    fn balance_to_u32_option(&self, input: BalanceOf<T>) -> Option<u32> {
      input.try_into().ok()
    }

   //  pub fn vote_proposal(&self, house_id: StorageIndex, house_owner_account: AccountIdOf<T>, proposal_id: StorageIndex, status: bool) -> DispatchResultWithPostInfo {
    pub fn vote_proposal(&self, proposal_id: StorageIndex, status: bool) -> DispatchResultWithPostInfo {
      
      // // Check if the proposal exist
      ensure!(Proposals::<T>::contains_key(proposal_id), Error::<T>::InvalidIndex);
      
      let proposal = Proposals::<T>::get(proposal_id).unwrap();
      ensure!(proposal.active == true, Error::<T>::ProposalOutDated);

      // Check if the account has a share in the contribution to be able to vote
      ensure!(Contributions::<T>::contains_key(self.account_id.clone()) == true, Error::<T>::ContributionNotExists);

      // Check if a vote already exist for this account in this proposal
      ensure!(Votes::<T>::contains_key(proposal_id, self.account_id.clone()) == false, Error::<T>::AlreadyVotedProposal);      

      // Create the vote
      let vote_id = <VoteIndex<T>>::get() + 1;
      <VoteIndex<T>>::put(vote_id);
      let block_number = <frame_system::Pallet<T>>::block_number();

      let vote = Vote {
         id: vote_id,
         account_id: self.account_id.clone(),
         status: status,
         timestamp: block_number
       };

      <Votes<T>>::insert(proposal_id, self.account_id.clone(), vote);

      let mut vote_ok_count = proposal.vote_ok_count;
      let mut vote_ko_count = proposal.vote_ko_count;
      if status {
         vote_ok_count += 1;
      }else {
         vote_ko_count += 1;
      }

      let newProposal = Proposal::<T>::new(proposal.id, proposal.house_id, proposal.account_id, 
         proposal.valuation, proposal.house_name, proposal.timestamp, 
         proposal.active, proposal.funded, vote_ok_count, vote_ko_count);

      Proposals::<T>::mutate(proposal_id, |item| {
         *item = Some(newProposal.clone());
      });

      Ok(().into())
    }
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct HouseOwner<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> HouseOwner<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !HouseOwners::<T>::contains_key(&account_id) {
         let house_owner = HouseOwner { account_id: account_id.clone() };
         HouseOwners::<T>::insert(account_id.clone(), house_owner);
         return true
      }
      false  
   }

   pub fn mint_house(&self, name: Vec<u8>) -> DispatchResultWithPostInfo {
      /// TODO: check if is still possible to mint a new house
      // Get a new house Id
      let house_id = <HouseIndex<T>>::get() + 1;
      <HouseIndex<T>>::put(house_id);

      // Create ownership relation
      let ownership_id = <OwnershipIndex<T>>::get() + 1;
      let _ownership_id = ownership_id;
      <OwnershipIndex<T>>::put(ownership_id);
      let block_number = <frame_system::Pallet<T>>::block_number();

      let ownership = Ownership {
         id: ownership_id.clone(),
         house_id: house_id.clone(),
         account_id: self.account_id.clone(),
         share: 100000,
         active: true,
         timestamp: block_number
      };

      // Ownerships::<T>::insert((house_id.clone(), self.account_id.clone()), ownership_id.clone(), ownership.clone());
      Ownerships::<T>::insert(ownership_id.clone(), ownership.clone());

      let mut house = HouseMinted::new(house_id.clone(), 1, name.clone(), block_number);
      house.ownerships.push(_ownership_id);

      MintedHouses::<T>::insert(house_id.clone(), house);

      Ok(().into())
   }

   pub fn create_proposal(&self, house_id: StorageIndex, valuation: BalanceOf<T>) -> DispatchResultWithPostInfo {

      // Check if the house is owned by the account
      ensure!(MintedHouses::<T>::contains_key(house_id), Error::<T>::InvalidIndex);

      let house = MintedHouses::<T>::get(house_id.clone()).unwrap();
      
      let _account_id = self.account_id.clone();

      let ownership = Ownerships::<T>::get(house.ownerships[0]).unwrap();
      ensure!(ownership.account_id == _account_id, Error::<T>::NotOwnedHouse);

      let mut wrap_proposalbis_iter = Proposals::<T>::iter();
      let exist_active_proposalbis = wrap_proposalbis_iter.position(|item| { 
         item.1.house_id ==  house_id && 
         item.1.account_id == self.account_id &&
         item.1.active == true });
      ensure!(exist_active_proposalbis.is_none() == true, Error::<T>::AlreadyActiveProposal);

      let is_valuation_validated = Pallet::<T>::validate_proposal_amount(valuation);

      ensure!(is_valuation_validated == true, Error::<T>::ProposalExceedFundLimit);

      // Create the proposal
      let block_number = <frame_system::Pallet<T>>::block_number();
      let proposal_id = <ProposalIndex<T>>::get() + 1;
      <ProposalIndex<T>>::put(proposal_id);
      let proposal = Proposal::new(proposal_id, house_id.clone(), self.account_id.clone(), 
         valuation, house.name.clone(), block_number, 
         true, false, 0, 0);
      
      <Proposals<T>>::insert(proposal_id, proposal);

      Ok(().into())
   }
}

pub struct EngineProcessor<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> EngineProcessor<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn manage_proposal(&self, house_id: StorageIndex,
      house_owner_account: AccountIdOf<T>, 
      proposal_id: StorageIndex
   ) -> DispatchResultWithPostInfo {

      let wrap_proposal = Proposals::<T>::get(proposal_id.clone());
      // Check if the proposal exist
      ensure!(wrap_proposal.is_none() == false, Error::<T>::InvalidIndex);
      let mut proposal =  wrap_proposal.unwrap();
      // Check if the proposal is still active
      ensure!(proposal.active == true, Error::<T>::ProposalOutDated);

      ensure!(house_id.clone() == proposal.house_id, Error::<T>::InvalidIndex);

      // We retrieve the votes to count them and determine if the yes has won
      let mut votes_iter = Votes::<T>::iter().filter(|item| item.0 == proposal_id);
      
      let mut votes_ok = Votes::<T>::iter().filter(|item| {
         let vote = item.2.clone();
         item.0 == proposal_id && vote.status == true
      });
      let votes_ko = Votes::<T>::iter().filter(|item| {
         let vote = item.2.clone();
         item.0 == proposal_id && vote.status == false
      });

      let total_votes_count_f = votes_iter.count() as f64;
      let votes_ok_count_f = votes_ok.count() as f64;
      let votes_ok_percentage = votes_ok_count_f / total_votes_count_f * 100.0; 

      let propo = items::Proposal::<T>::new(proposal.id, proposal.house_id, proposal.account_id.clone(), 
         proposal.valuation, proposal.house_name, proposal.timestamp, false, 
         true, proposal.vote_ok_count, proposal.vote_ko_count);
         
      Proposals::<T>::mutate(proposal_id, |val| {
         *val = Some(propo);
      });

      // let votes_ok_percentage = 52.0;
      if votes_ok_percentage > 51.0 {

         // // We update the house_owner ownership of the house
         let house = MintedHouses::<T>::get(house_id).unwrap();

         let house_ownership_id = house.ownerships.get(0).unwrap();

         let mut wrap_house_ownership = Ownerships::<T>::get(house_ownership_id);

         let mut house_ownership = wrap_house_ownership.unwrap();

         let mut new_house_ownership = Ownership::<T>::new(*house_ownership_id, house_id, house_ownership.account_id.clone(), house_ownership.share, house_ownership.timestamp, false);

         Ownerships::<T>::mutate(house_ownership.id, |val| {
            *val = Some(new_house_ownership.clone());
         });

         // Get the ok votes of the investissors having a current contribution > 0
         let votes_ok_list = Votes::<T>::iter().filter(|val| {
            let vote = val.2.clone();            
            val.0 == proposal_id && vote.status == true && Contributions::<T>::contains_key(self.account_id.clone()) == true
         });

         let mut investissor_iter = votes_ok_list.map(|val| val.1);

         // We calculate the percentage of the yes votes
         let mut voting_power: u32 = 0;
         for item in investissor_iter {
            let contribution = Contributions::<T>::get(item).unwrap();
            voting_power += contribution.1;
         }

         let mut ownerships = Vec::new();
         let block_number = <frame_system::Pallet<T>>::block_number();

         // We refresh the iterator of the ok votes
         let votes_ok_list_a = Votes::<T>::iter().filter(|val| {
            let vote = val.2.clone();
            val.0 == proposal_id && vote.status == true
         });

         // We retrieve the AccountId of the yes votes
         let mut investissor_iter_a = votes_ok_list_a.map(|val| val.1);

         for item in investissor_iter_a {

            let contribution = Contributions::<T>::get(item.clone()).unwrap();
            let ownership_id = <OwnershipIndex<T>>::get() + 1;

            // We calculate the share of the account for the house
            let new_share = contribution.1 * 100000 / voting_power;

            let new_ownership = Ownership::<T>::new(ownership_id.clone(), house_id, item.clone(), new_share, block_number, true);

            <OwnershipIndex<T>>::put(ownership_id.clone());
            Ownerships::<T>::insert(ownership_id.clone(), new_ownership.clone());
            ownerships.push(ownership_id);
         }

         let mut new_house = HouseMinted::<T, NftIndex>::new(house_id, house.nft, house.name, block_number);
         new_house.ownerships = ownerships;

         MintedHouses::<T>::remove(house_id);
         FSHouses::<T>::insert(house_id, new_house);
      }

      Ok(().into())
   }
}


#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Tenant<T: Config> {
   pub account_id: AccountIdOf<T>
}
impl<T: Config> Tenant<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id
      }
   }

   pub fn create(account_id: AccountIdOf<T>) -> bool {
      if !Tenants::<T>::contains_key(&account_id) {
         let tenant = Tenant { account_id: account_id.clone() };
         Tenants::<T>::insert(account_id.clone(), tenant);
         return true
      }
      false   
   }
}

