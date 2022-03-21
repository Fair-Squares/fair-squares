pub use super::*;
//use frame_support::inherent::Vec;
use scale_info::TypeInfo;
//use frame_support::storage::StorageValue;


pub type Something<T> = StorageValue<T, u32>;


pub struct House{
    pub house_nft: u32,
    pub houseowner: u32,
    pub valuation: f32
}

pub struct HouseFS<T, U> {
   pub house_nft: T,
   pub valuation: u32,
   pub ownerships: Vec<Ownership<U>>,
}
impl<T,U> HouseFS<T,U> {
   pub fn new(house_nft: T, valuation: u32) -> Self {
      Self {
         house_nft,
         valuation,
         ownerships: Vec::<Ownership<U>>::new()
      }
   }
}

pub struct Proposal<T> {
   //pub house_owner: T,
   pub house: House,
   pub valuation: f32,
   pub votes: Vec<Vote<T>>
}
impl<T> Proposal<T> {
   pub fn new(house: House, valuation: f32) -> Self {
      Self {
         house,
         valuation,
         votes: Vec::<Vote<T>>::new()
      }
   }
   
   pub fn add_vote(&mut self, vote: Vote<T>) {
      self.votes.push(vote)
   } 
}


#[derive(Clone, Encode, Decode, Default, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Contribution<T> {
    pub investor:T,
    pub amount:u32,
}
impl<T>Contribution<T> {
    pub fn new(investor:T, amount: u32)-> Self {
        Self {
            investor,
            amount,
        }
    }
}

pub enum VoteStatus {
   Aprouved,
   Rejected
}

pub struct Vote<T> {
   pub investor: T,
   pub status: VoteStatus
}

impl<T>Vote<T> {
   pub fn new(investor: T, status: VoteStatus) -> Self {
      Self {
         investor,
         status
      }
   }
}

pub struct Ownership<T> {
  pub investor: T,
  pub percentage: f32
}

impl<T>Ownership<T> {
   pub fn new(investor: T, percentage: f32) -> Self {
      Self {
         investor,
         percentage
      }
   }
}
