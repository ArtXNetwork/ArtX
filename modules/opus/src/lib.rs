#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, StorageValue, StorageMap, Parameter, ensure,
    dispatch::DispatchResult,
    weights::SimpleDispatchInfo,
};
use sp_runtime::traits::{ Member };
use frame_system::{self as system, ensure_signed};
use codec::{Encode, Decode};
use sp_std::prelude::*;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type ContentHash: Parameter + Member + Default + Copy;
    type OpusType: Parameter + Member + Default + Copy;
    type Topic: Parameter + Member + Default + Copy;
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Opus<ContentHash, OpusType, Topic> {
    pub id: ContentHash,
    pub opus_type: OpusType,
    pub topic: Topic,
    pub sources: Vec<ContentHash>
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Opus {
    Opuses get(fn opuses): map hasher(blake2_128_concat) T::ContentHash => Option<Opus<T::ContentHash, T::OpusType, T::Topic>>;
    OpusOwner get(fn owner_of): map hasher(blake2_128_concat) T::ContentHash => Option<T::AccountId>;

    IndexOpus get(fn opus_by_index): map hasher(blake2_128_concat) u64 => T::ContentHash;
    OpusCount get(fn opus_count): u64;
    OpusIndex: map hasher(blake2_128_concat) T::ContentHash => u64;

    OwnedIndexOpus get(fn owned_opus_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::ContentHash;
    OwnedOpusCount get(fn owned_opus_count): map hasher(blake2_128_concat) T::AccountId => u64;
    OwnedOpusIndex: map hasher(blake2_128_concat) T::ContentHash => u64;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events.
    fn deposit_event() = default;

    #[weight = SimpleDispatchInfo::default()]
    pub fn create(origin, content_hash: T::ContentHash, opus_type: T::OpusType, topic: T::Topic, sources: Vec<T::ContentHash>) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      ensure!(!<OpusOwner<T>>::contains_key(content_hash), "Opus already exists");
      ensure!(sources.len() <= 10, "Cannot link more than 10 sources");

      let new_opus = Opus {
          id: content_hash,
          opus_type,
          topic,
          sources: sources.clone(),
      };

      let new_opus_count = Self::opus_count() + 1;

      let new_owned_opus_count = Self::owned_opus_count(sender.clone()) + 1;

      <Opuses<T>>::insert(content_hash, new_opus);
      <OpusOwner<T>>::insert(content_hash, sender.clone());

      <IndexOpus<T>>::insert(new_opus_count, content_hash);
      OpusCount::put(new_opus_count);
      <OpusIndex<T>>::insert(content_hash, new_opus_count);

      <OwnedIndexOpus<T>>::insert((sender.clone(), new_owned_opus_count), content_hash);
      <OwnedOpusCount<T>>::insert(sender.clone(), new_owned_opus_count);
      <OwnedOpusIndex<T>>::insert(content_hash, new_owned_opus_count);

      Self::deposit_event(RawEvent::Created(sender, content_hash, opus_type, topic, sources));

      Ok(())
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn transfer(origin, to: T::AccountId, content_hash: T::ContentHash) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let owner = Self::owner_of(content_hash).ok_or("No opus owner")?;

      ensure!(owner == sender.clone(), "Sender does not own the opus");

      let owned_opus_count_from = Self::owned_opus_count(sender.clone());
      let owned_opus_count_to = Self::owned_opus_count(to.clone());

      let new_owned_opus_count_to = owned_opus_count_to.checked_add(1)
        .ok_or("Transfer causes overflow for opus receiver")?;

      let new_owned_opus_count_from = owned_opus_count_from.checked_sub(1)
          .ok_or("Transfer causes underflow for opus sender")?;

      let owned_opus_index = <OwnedOpusIndex<T>>::get(content_hash);
      if owned_opus_index != new_owned_opus_count_from {
        let last_owned_opus_id = <OwnedIndexOpus<T>>::get((sender.clone(), new_owned_opus_count_from));
        <OwnedIndexOpus<T>>::insert((sender.clone(), owned_opus_index), last_owned_opus_id);
        <OwnedOpusIndex<T>>::insert(last_owned_opus_id, owned_opus_index);
      }

      <OpusOwner<T>>::insert(content_hash, to.clone());
      <OwnedOpusIndex<T>>::insert(content_hash, owned_opus_count_to);

      <OwnedIndexOpus<T>>::remove((sender.clone(), new_owned_opus_count_from));
      <OwnedIndexOpus<T>>::insert((to.clone(), owned_opus_count_to), content_hash);

      <OwnedOpusCount<T>>::insert(sender.clone(), new_owned_opus_count_from);
      <OwnedOpusCount<T>>::insert(to.clone(), new_owned_opus_count_to);

      Self::deposit_event(RawEvent::Transferred(sender, to, content_hash));

      Ok(())
    }

  }
}

decl_event!(
  pub enum Event<T>
  where
    AccountId = <T as system::Trait>::AccountId,
    ContentHash = <T as Trait>::ContentHash,
    OpusType = <T as Trait>::OpusType,
    Topic = <T as Trait>::Topic,
    VecContentHash = Vec<<T as Trait>::ContentHash>,
  {
    Created(AccountId, ContentHash, OpusType, Topic, VecContentHash),
    Transferred(AccountId, AccountId, ContentHash),
  }
);
