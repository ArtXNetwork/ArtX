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
    type OpusId: Parameter + Member + Default + Copy;
    type OpusType: Parameter + Member + Default + Copy;
    type Topic: Parameter + Member + Default + Copy;
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Opus<OpusId, OpusType, Topic> {
    pub opus_type: OpusType,
    pub topic: Topic,
    pub sources: Vec<OpusId>
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Opus {
    Opuses get(fn opuses): map hasher(blake2_128_concat) T::OpusId => Option<Opus<T::OpusId, T::OpusType, T::Topic>>;
    OpusOwner get(fn owner_of): map hasher(blake2_128_concat) T::OpusId => Option<T::AccountId>;

    IndexOpus get(fn opus_by_index): map hasher(blake2_128_concat) u64 => T::OpusId;
    OpusCount get(fn opus_count): u64;
    OpusIndex: map hasher(blake2_128_concat) T::OpusId => u64;

    OwnedIndexOpus get(fn owned_opus_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::OpusId;
    OwnedOpusCount get(fn owned_opus_count): map hasher(blake2_128_concat) T::AccountId => u64;
    OwnedOpusIndex: map hasher(blake2_128_concat) T::OpusId => u64;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events.
    fn deposit_event() = default;

    #[weight = SimpleDispatchInfo::default()]
    pub fn create(origin, opus_id: T::OpusId, opus_type: T::OpusType, topic: T::Topic, sources: Vec<T::OpusId>) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      ensure!(!<OpusOwner<T>>::contains_key(opus_id), "Opus already exists");
      ensure!(sources.len() <= 10, "Cannot link more than 10 sources");

      let new_opus = Opus {
          opus_type,
          topic,
          sources: sources.clone(),
      };

      let new_opus_count = Self::opus_count() + 1;

      let new_owned_opus_count = Self::owned_opus_count(sender.clone()) + 1;

      <Opuses<T>>::insert(opus_id, new_opus);
      <OpusOwner<T>>::insert(opus_id, sender.clone());

      <IndexOpus<T>>::insert(new_opus_count, opus_id);
      OpusCount::put(new_opus_count);
      <OpusIndex<T>>::insert(opus_id, new_opus_count);

      <OwnedIndexOpus<T>>::insert((sender.clone(), new_owned_opus_count), opus_id);
      <OwnedOpusCount<T>>::insert(sender.clone(), new_owned_opus_count);
      <OwnedOpusIndex<T>>::insert(opus_id, new_owned_opus_count);

      Self::deposit_event(RawEvent::Created(sender, opus_id, opus_type, topic, sources));

      Ok(())
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn transfer(origin, to: T::AccountId, opus_id: T::OpusId) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let owner = Self::owner_of(opus_id).ok_or("No opus owner")?;

      ensure!(owner == sender.clone(), "Sender does not own the opus");

      let owned_opus_count_from = Self::owned_opus_count(sender.clone());
      let owned_opus_count_to = Self::owned_opus_count(to.clone());

      let new_owned_opus_count_to = owned_opus_count_to.checked_add(1)
        .ok_or("Transfer causes overflow for opus receiver")?;

      let new_owned_opus_count_from = owned_opus_count_from.checked_sub(1)
        .ok_or("Transfer causes underflow for opus sender")?;

      let owned_opus_index = <OwnedOpusIndex<T>>::get(opus_id);
      if owned_opus_index != new_owned_opus_count_from {
        let last_owned_opus_id = <OwnedIndexOpus<T>>::get((sender.clone(), new_owned_opus_count_from));
        <OwnedIndexOpus<T>>::insert((sender.clone(), owned_opus_index), last_owned_opus_id);
        <OwnedOpusIndex<T>>::insert(last_owned_opus_id, owned_opus_index);
      }

      <OpusOwner<T>>::insert(opus_id, to.clone());
      <OwnedOpusIndex<T>>::insert(opus_id, owned_opus_count_to);

      <OwnedIndexOpus<T>>::remove((sender.clone(), new_owned_opus_count_from));
      <OwnedIndexOpus<T>>::insert((to.clone(), owned_opus_count_to), opus_id);

      <OwnedOpusCount<T>>::insert(sender.clone(), new_owned_opus_count_from);
      <OwnedOpusCount<T>>::insert(to.clone(), new_owned_opus_count_to);

      Self::deposit_event(RawEvent::Transferred(sender, to, opus_id));

      Ok(())
    }

  }
}

decl_event!(
  pub enum Event<T>
  where
    AccountId = <T as system::Trait>::AccountId,
    OpusId = <T as Trait>::OpusId,
    OpusType = <T as Trait>::OpusType,
    Topic = <T as Trait>::Topic,
    VecOpusId = Vec<<T as Trait>::OpusId>,
  {
    Created(AccountId, OpusId, OpusType, Topic, VecOpusId),
    Transferred(AccountId, AccountId, OpusId),
  }
);
