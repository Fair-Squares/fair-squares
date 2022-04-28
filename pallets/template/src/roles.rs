
pub use super::*;
// mod items;
pub use crate::items::*;
//pub use frame_support::dispatch::DispatchResult;
pub use frame_support::{
   dispatch::DispatchResult,
   pallet_prelude::*,
   codec::{Encode, Decode},
   traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, WithdrawReasons}
};
pub use frame_system::{pallet_prelude::*,ensure_signed};
use frame_support::inherent::Vec;

use scale_info::TypeInfo;

pub type StorageIndex = u32;
pub type NftIndex = u32;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;



pub struct Investor<T: Config> {
    pub account_id: AccountIdOf<T>,
}
impl<T: Config> Investor<T>{
    pub fn new(account_id: AccountIdOf<T>)-> Self{
        Self {
            account_id
        }        
    }

    pub fn add_contribution_fund(&self, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
      
      ensure!(amount >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
      let _account = self.account_id.clone();
      let block_number = <frame_system::Pallet<T>>::block_number();
      let contribution = Contribution {
         amount: amount,
         timestamp: block_number
      };

      let fund = <FundAmount<T>>::get();
      let total_fund = fund + amount.clone();

      if !Contributions::<T>::contains_key(&_account) {
         // let share = amount.clone() / total_fund;
         // let converted_share = TryInto::<u32>::try_into(share).ok();

         // let mut fund_share= match converted_share {
         //    Some(x) => x * 100,
         //    None => 0,
         // };

         //let sharing_fund = FundSharing { amount: amount, share: converted_share.unwrap() * 100 };

         Contributions::<T>::insert(&_account, (amount, 0));

         let mut contribution_list = Vec::new();
         contribution_list.push(contribution);
         ContributionLog::<T>::insert(&_account, contribution_list);
      } else {
         ContributionLog::<T>::mutate(&_account, |val| {
            val.push(contribution);
         });

         Contributions::<T>::mutate(&_account, |val| {
             val.0 += amount;
         });
      }

      <FundAmount<T>>::put(total_fund);

      // Update the share for all contributors
      for mut item in Contributions::<T>::iter() {
         
         let share = item.1.0.clone() / total_fund;
         let converted_share = TryInto::<u32>::try_into(share).ok();

         let fund_share= match converted_share {
            Some(x) => x * 100,
            None => 0,
         };

         Contributions::<T>::mutate(item.0, |val| {
            val.1 = fund_share;
        });
      }

      Ok(().into())
    }

    pub fn vote_proposal(&self, house_id: StorageIndex, house_owner_account: AccountIdOf<T>, proposal_id: StorageIndex, status: bool) -> DispatchResultWithPostInfo {
      
      // Check if the proposal exist
      let proposal_exist = Proposals::<T>::contains_key((house_id, house_owner_account.clone()), proposal_id);
      ensure!(proposal_exist == true, Error::<T>::InvalidIndex);
      
      let proposal = Proposals::<T>::get((house_id, house_owner_account.clone()), proposal_id).unwrap();
      ensure!(proposal.active == true, Error::<T>::ProposalOutDated);

      // Check if a vote already exist for this account in this proposal
      ensure!(Votes::<T>::contains_key(proposal_id, self.account_id.clone()) == false, Error::<T>::AlreadyVotedProposal);

      // Check if the account has a share in the contribution to be able to vote
      ensure!(Contributions::<T>::contains_key(self.account_id.clone()) == true, Error::<T>::ContributionNotExists);

      // Create the vote
      let vote_id = <VoteIndex<T>>::get();
      <VoteIndex<T>>::put(vote_id + 1);
      let block_number = <frame_system::Pallet<T>>::block_number();

      let vote = Vote {
         id: vote_id,
         account_id: self.account_id.clone(),
         status: status,
         timestamp: block_number
       };

      <Votes<T>>::insert(proposal_id, self.account_id.clone(), vote);

      Ok(().into())
    }
}

pub struct HouseOwnerBis<T: Config> {
   pub account_id: AccountIdOf<T>,
   pub houses: Vec<StorageIndex>
}
impl<T: Config> HouseOwnerBis<T> {
   pub fn new(account_id: AccountIdOf<T>) -> Self {
      Self {
         account_id,
         houses: Vec::<StorageIndex>::new()
      }
   }

   pub fn mint_house(&self) -> DispatchResultWithPostInfo {
      /// TODO: check if is still possible to mint a new house
      // Get a new house Id
      let house_id = <HouseIndexBis<T>>::get();
      <HouseIndexBis<T>>::put(house_id + 1);

      // Create ownership relation
      let ownership_id = <OwnershipIndex<T>>::get();
      let _ownership_id = ownership_id;
      <OwnershipIndex<T>>::put(ownership_id + 1);
      let block_number = <frame_system::Pallet<T>>::block_number();

      let ownership = Ownership {
         id: ownership_id,
         house_id: house_id,
         account_id: self.account_id.clone(),
         share: 100,
         active: true,
         timestamp: block_number
      };

      Ownerships::<T>::insert((house_id, self.account_id.clone()), ownership_id, ownership);

      let mut house = HouseMinted::new(house_id, 1, block_number);
      house.ownerships.push(_ownership_id);

      MintedHouses::<T>::insert(house_id, house);

      Ok(().into())
   }

   pub fn create_proposal(&self, house_id: StorageIndex, valuation: u32) -> DispatchResultWithPostInfo {

      // Check if the house is owned by the account
      
      let house = MintedHouses::<T>::get(house_id).unwrap();
      let house_ownerships_iter = house.ownerships.iter();
      
      let _account_id = self.account_id.clone();

      let mut wrap_ownership_iter = Ownerships::<T>::iter_prefix_values((house_id, _account_id));
      let exist_ownership = wrap_ownership_iter.position(|item| {
         item.active == true
      });
      ensure!(exist_ownership.is_none() == true, Error::<T>::NotOwnedHouse);

      // Check if there is already a current proposal for this house
      let mut wrap_proposal_iter = Proposals::<T>::iter_prefix_values((house_id, self.account_id.clone()));
      let exist_active_proposal = wrap_proposal_iter.position(|val| val.active == true);
      ensure!(exist_active_proposal.is_none() == false, Error::<T>::AlreadyActiveProposal);

      // Create the proposal
      let block_number = <frame_system::Pallet<T>>::block_number();
      let proposal_id = <ProposalIndex<T>>::get();
      <ProposalIndex<T>>::put(proposal_id + 1);
      let proposal = Proposal::new(proposal_id, house_id, self.account_id.clone(), valuation, block_number, true, false);
      
      <Proposals<T>>::insert((house_id, self.account_id.clone()), proposal_id, proposal);

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

      let wrap_proposal = Proposals::<T>::get((house_id, house_owner_account.clone()), proposal_id);
      // Check if the proposal exist
      ensure!(wrap_proposal.is_none() == false, Error::<T>::InvalidIndex);
      let mut proposal =  wrap_proposal.unwrap();
      // Check if the proposal is still active
      ensure!(proposal.active == true, Error::<T>::ProposalOutDated);

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
      
      // We update the proposal with the active field to false and the funded flaag according to the result of the vote
      let propo = items::Proposal::<T>::new(proposal.id, proposal.house_id, proposal.account_id.clone(), proposal.valuation, proposal.timestamp, false, votes_ok_percentage > 51.0);
 
      Proposals::<T>::mutate((house_id, house_owner_account.clone()), proposal_id, |val| {
        *val = Some(propo);
     });

      if votes_ok_percentage > 51.0 {

         // We update the house_owner ownership of the house
         let house = MintedHouses::<T>::get(house_id).unwrap();

         let house_ownership_id = house.ownerships.get(0).unwrap();

         let mut wrap_house_ownership = Ownerships::<T>::get((house_id, house_owner_account.clone()), house_ownership_id);
         let mut house_ownership = wrap_house_ownership.unwrap();

         let mut new_house_ownership = Ownership::<T>::new(*house_ownership_id, house_id, house_ownership.account_id.clone(), house_ownership.share, house_ownership.timestamp, false);

         Ownerships::<T>::mutate((house_id, house_owner_account.clone()), house_ownership.id, |val| {
            *val = Some(new_house_ownership);
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
            let contribution = Contributions::<T>::get(item);
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

            let contribution = Contributions::<T>::get(item.clone());
            let ownership_id = <OwnershipIndex<T>>::get();

            // We calculate the share of the account for the house
            let new_share = contribution.1 * 100 / voting_power;

            let new_ownership = Ownership::<T>::new(ownership_id, house_id, item.clone(), new_share, block_number, true);

            <OwnershipIndex<T>>::put(ownership_id + 1);
            Ownerships::<T>::insert((house_id, house_owner_account.clone()), ownership_id, new_ownership);
            ownerships.push(ownership_id);
         }

         let mut new_house = HouseMinted::<T, NftIndex>::new(house_id, house.nft, block_number);
         new_house.ownerships = ownerships;

         MintedHouses::<T>::remove(house_id);
         FSHouses::<T>::insert(house_id, new_house);
      }

      Ok(().into())
   }
}


