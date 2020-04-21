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
    type NodeType: Parameter + Member + Default + Copy;
    type NodeTopic: Parameter + Member + Default + Copy;
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Node<ContentHash, NodeType, NodeTopic> {
    pub id: ContentHash,
    pub node_type: NodeType,
    pub node_topic: NodeTopic,
    pub sources: Vec<ContentHash>
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Node {
    Nodes get(fn node): map hasher(blake2_128_concat) T::ContentHash => Option<Node<T::ContentHash, T::NodeType, T::NodeTopic>>;
    NodeOwner get(fn owner_of): map hasher(blake2_128_concat) T::ContentHash => Option<T::AccountId>;

    AllNodesArray get(fn node_by_index): map hasher(blake2_128_concat) u64 => T::ContentHash;
    AllNodesCount get(fn all_nodes_count): u64;
    AllNodesIndex: map hasher(blake2_128_concat) T::ContentHash => u64;

    OwnedNodesArray get(fn node_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::ContentHash;
    OwnedNodesCount get(fn owned_nodes_count): map hasher(blake2_128_concat) T::AccountId => u64;
    OwnedNodesIndex: map hasher(blake2_128_concat) T::ContentHash => u64;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events.
    fn deposit_event() = default;

    #[weight = SimpleDispatchInfo::default()]
    pub fn create(origin, content_hash: T::ContentHash, node_type: T::NodeType, node_topic: T::NodeTopic, sources: Vec<T::ContentHash>) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      ensure!(!<NodeOwner<T>>::contains_key(content_hash), "Content Node already exists");
      ensure!(sources.len() <= 10, "Cannot link more than 10 sources");
      let new_node = Node {
          id: content_hash,
          node_type,
          node_topic,
          sources: sources.clone(),
      };

      let owned_nodes_count = Self::owned_nodes_count(sender.clone());
      let new_owned_nodes_count = owned_nodes_count.checked_add(1)
          .ok_or("Exceed max node count per account")?;

      let all_nodes_count = Self::all_nodes_count();
      let new_all_nodes_count = all_nodes_count.checked_add(1)
          .ok_or("Exceed total max node count")?;

      <Nodes<T>>::insert(content_hash, new_node);
      <NodeOwner<T>>::insert(content_hash, sender.clone());

      <AllNodesArray<T>>::insert(new_all_nodes_count, content_hash);
      AllNodesCount::put(new_all_nodes_count);
      <AllNodesIndex<T>>::insert(content_hash, new_all_nodes_count);

      <OwnedNodesArray<T>>::insert((sender.clone(), new_owned_nodes_count), content_hash);
      <OwnedNodesCount<T>>::insert(sender.clone(), new_owned_nodes_count);
      <OwnedNodesIndex<T>>::insert(content_hash, new_owned_nodes_count);

      Self::deposit_event(RawEvent::Created(sender, content_hash, node_type, node_topic, sources));

      Ok(())
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn transfer(origin, to: T::AccountId, content_hash: T::ContentHash) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let owner = Self::owner_of(content_hash).ok_or("No node owner")?;

      ensure!(owner == sender.clone(), "Sender does not own the node");

      let owned_nodes_count_from = Self::owned_nodes_count(sender.clone());
      let owned_nodes_count_to = Self::owned_nodes_count(to.clone());

      let new_owned_nodes_count_to = owned_nodes_count_to.checked_add(1)
        .ok_or("Transfer causes overflow for node receiver")?;

      let new_owned_nodes_count_from = owned_nodes_count_from.checked_sub(1)
          .ok_or("Transfer causes underflow for node sender")?;

      let owned_node_index = <OwnedNodesIndex<T>>::get(content_hash);
      if owned_node_index != new_owned_nodes_count_from {
        let last_owned_node_id = <OwnedNodesArray<T>>::get((sender.clone(), new_owned_nodes_count_from));
        <OwnedNodesArray<T>>::insert((sender.clone(), owned_node_index), last_owned_node_id);
        <OwnedNodesIndex<T>>::insert(last_owned_node_id, owned_node_index);
      }

      <NodeOwner<T>>::insert(content_hash, to.clone());
      <OwnedNodesIndex<T>>::insert(content_hash, owned_nodes_count_to);

      <OwnedNodesArray<T>>::remove((sender.clone(), new_owned_nodes_count_from));
      <OwnedNodesArray<T>>::insert((to.clone(), owned_nodes_count_to), content_hash);

      <OwnedNodesCount<T>>::insert(sender.clone(), new_owned_nodes_count_from);
      <OwnedNodesCount<T>>::insert(to.clone(), new_owned_nodes_count_to);

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
    NodeType = <T as Trait>::NodeType,
    NodeTopic = <T as Trait>::NodeTopic,
    VecContentHash = Vec<<T as Trait>::ContentHash>,
  {
    Created(AccountId, ContentHash, NodeType, NodeTopic, VecContentHash),
    Transferred(AccountId, AccountId, ContentHash),
  }
);
