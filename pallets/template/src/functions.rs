
pub use crate::items::*;

use frame_support::{
    sp_runtime::traits::{Hash},
    storage::child
 };
use frame_support::inherent::Vec;
use scale_info::{ prelude::vec };


 
impl<T: Config> Pallet<T> {
   
      /// Each fund stores information about its contributors and their contributions in a child trie
      // This helper function calculates the id of the associated child trie.
      pub fn id_from_index() -> child::ChildInfo {
         let mut buf = Vec::new();
         buf.extend_from_slice(b"treasury");
         //buf.extend_from_slice(&index.to_le_bytes()[..]);

         child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
      }
      
      /// Record a contribution in the associated child trie.
      pub fn contribution_put( who: &T::AccountId, balance: &BalanceOf<T>) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::put(&id, b, &balance));
      }
   
      /// Lookup a contribution in the associated child trie.
      pub fn contribution_get(who: &T::AccountId) -> BalanceOf<T> {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
      }
      
      /// Remove a contribution from an associated child trie.
      pub fn contribution_kill(who: &T::AccountId) {
         let id = Self::id_from_index();
         who.using_encoded(|b| child::kill(&id, b));
      }

      pub fn set_roles(account: AccountIdOf<T>) -> DispatchResultWithPostInfo {

         let _account = account.clone();
         // if !Roles::<T>::contains_key(&_account) {
         //    let role = Role { roles : vec![INVESTOR_ROLE, HOUSE_OWNER_ROLE]};
         //    let wrap_rop = vec![role];
         //    Roles::<T>::insert(&_account, wrap_rop);
         // }

         if !Roles::<T>::contains_key(&_account) {
            let roles = vec![INVESTOR_ROLE, HOUSE_OWNER_ROLE];
            Roles::<T>::insert(&_account, roles);
         }

         Ok(().into())
      }
   }