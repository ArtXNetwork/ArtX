#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, StorageValue, StorageMap, Parameter, ensure,
    dispatch::{DispatchResult, DispatchError},
    traits::{Currency, ReservableCurrency, ExistenceRequirement},
    weights::SimpleDispatchInfo,
};
use sp_runtime::traits::{
    Member, AtLeast32Bit, Bounded, CheckedAdd, CheckedMul, One,
};
use frame_system::{self as system, ensure_signed};
use codec::{Encode, Decode};
use opus;

/// The module's configuration trait.
pub trait Trait: system::Trait + opus::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type LikeId: Parameter + Member + Default + Bounded + AtLeast32Bit + Copy;
    type CollectId: Parameter + Member + Default + Bounded + AtLeast32Bit + Copy;
    type GrantId: Parameter + Member + Default + Bounded + AtLeast32Bit + Copy;
    type ShareId: Parameter + Member + Default + Bounded + AtLeast32Bit + Copy;
    type ReportId: Parameter + Member + Default + Bounded + AtLeast32Bit + Copy;
    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>;
}
// Balance zone
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Like<AccountId, ContentHash> {
    pub from: AccountId,
    pub to: ContentHash,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Collect<AccountId, ContentHash> {
    pub from: AccountId,
    pub to: ContentHash,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Share<AccountId, ContentHash> {
    pub from: AccountId,
    pub to: ContentHash,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Grant<AccountId, ContentHash, Balance> {
    pub from: AccountId,
    pub to: ContentHash,
    pub amount: Balance,
}

#[cfg_attr(feature ="std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Report<ContentHash, AccountId> {
    pub from: AccountId,
    pub target: ContentHash,
    pub reason: ContentHash,
}

// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as Interaction {
    pub Likes get(fn likes): map hasher(blake2_128_concat) T::LikeId => Option<Like<T::AccountId, <T as opus::Trait>::ContentHash>>;
    pub NextLikeId get(fn next_like_id): T::LikeId;

    pub Collects get(fn collects): map hasher(blake2_128_concat) T::CollectId => Option<Collect<T::AccountId, <T as opus::Trait>::ContentHash>>;
    pub NextCollectId get(fn next_collect_id): T::CollectId;

    pub Grants get(fn grants): map hasher(blake2_128_concat) T::GrantId => Option<Grant<T::AccountId, <T as opus::Trait>::ContentHash, BalanceOf<T>>>;
    pub NextGrantId get(fn next_grant_id): T::GrantId;

    pub Reports get(fn reports): map hasher(blake2_128_concat) T::ReportId => Option<Report<<T as opus::Trait>::ContentHash, T::AccountId>>;
    pub NextReportId get(fn next_report_id): T::ReportId;

    // how many people like this opus
    pub OpusLikedCount get(fn opus_liked_count): map hasher(blake2_128_concat) <T as opus::Trait>::ContentHash => u64;
    // how many people collect this opus
    pub OpusCollectedCount get(fn opus_collected_count): map hasher(blake2_128_concat) <T as opus::Trait>::ContentHash => u64;
    // how much money this opus received
    pub OpusGrantedBalance get(fn opus_granted_balance): map hasher(blake2_128_concat) <T as opus::Trait>::ContentHash => BalanceOf<T>;
    // who report this opus
    pub OpusReportedBy get(fn opus_reported_by): map hasher(blake2_128_concat) <T as opus::Trait>::ContentHash => T::AccountId;

    pub LikedOpus get(fn liked_opus): map hasher(blake2_128_concat) (T::AccountId, <T as opus::Trait>::ContentHash) => T::LikeId;
    pub CollectedOpus get(fn collected_opus): map hasher(blake2_128_concat) (T::AccountId, <T as opus::Trait>::ContentHash) => T::CollectId;
    pub GrantedOpus get(fn granted_opus): map hasher(blake2_128_concat) (T::AccountId, <T as opus::Trait>::ContentHash) => T::GrantId;
    pub ReportedOpus get(fn reported_opus): map hasher(blake2_128_concat) (T::AccountId, <T as opus::Trait>::ContentHash) => T::ReportId;
  }
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    #[weight = SimpleDispatchInfo::default()]
    pub fn like(origin, to: <T as opus::Trait>::ContentHash) -> Result<(), DispatchError> {
      let sender = ensure_signed(origin)?;
      <opus::Module<T>>::owner_of(to).ok_or("Opus does not exist")?;
      ensure!(<LikedOpus<T>>::contains_key((sender.clone(), to)), "Already liked this Opus.");

      Self::do_like(sender.clone(), to)
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn collect(origin, to: <T as opus::Trait>::ContentHash) -> Result<(), DispatchError> {
      let sender = ensure_signed(origin)?;
      let opus_owner = match <opus::Module<T>>::owner_of(to) {
        Some(owner) => owner,
        None => return Err(DispatchError::Other("Opus does not exist")),
      };
      ensure!(sender != opus_owner, "Can not collect yourself.");

      ensure!(<CollectedOpus<T>>::contains_key((sender.clone(), to)), "Already collected this Opus.");

      Self::do_collect(sender.clone(), opus_owner, to)
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn grant(origin, to: <T as opus::Trait>::ContentHash, amount: BalanceOf<T>) -> Result<(), DispatchError> {
      let sender = ensure_signed(origin)?;
      let opus_owner = match <opus::Module<T>>::owner_of(to) {
        Some(owner) => owner,
        None => return Err(DispatchError::Other("Opus does not exist")),
      };
      ensure!(sender != opus_owner, "Can not grant yourself.");
      ensure!(<GrantedOpus<T>>::contains_key((sender.clone(), to)), "Already granted this Opus.");

      ensure!(amount > BalanceOf::<T>::from(0), "grant should more than 0.");
      Self::do_grant(sender.clone(), opus_owner, to, amount)
    }

    #[weight = SimpleDispatchInfo::default()]
    pub fn report(origin, target: <T as opus::Trait>::ContentHash, reason: <T as opus::Trait>::ContentHash) -> Result<(), DispatchError> {
      let sender = ensure_signed(origin)?;
      // <opus::Module<T>>::owner_of(target).ok_or("Content Opus does not exist")?;
      let opus_owner = match <opus::Module<T>>::owner_of(target) {
        Some(owner) => owner,
        None => return Err(DispatchError::Other("Opus does not exist")),
      };
      ensure!(sender != opus_owner, "Can not report yourself.");
      ensure!(<ReportedOpus<T>>::contains_key((sender.clone(), target)), "Already reported this Opus.");

      Self::do_report(sender.clone(), target, reason)
    }
  }
}

decl_event!(
  pub enum Event<T>
  where
    AccountId = <T as system::Trait>::AccountId,
    ContentHash = <T as opus::Trait>::ContentHash,
    Amount = BalanceOf<T>,
    LikeId = <T as Trait>::LikeId,
    CollectId = <T as Trait>::CollectId,
    GrantId = <T as Trait>::GrantId,
    ReportId = <T as Trait>::ReportId,
    ReasonHash = <T as opus::Trait>::ContentHash,
  {
    Liked(ContentHash, AccountId, LikeId, u64),
    /// (ContentHash, sender, receiver, CollectId, collected_count)
    Collected(ContentHash, AccountId, AccountId, CollectId, u64),
    /// (ContentHash, sender, receiver, GrantedAmount, GrantId, OpusGrantedBalance)
    Granted(ContentHash, AccountId, AccountId, Amount, GrantId, Amount),
    Reported(ContentHash, AccountId, ReasonHash, ReportId),
  }
);

impl<T: Trait> Module<T> {
    pub fn do_like(sender: T::AccountId, to: <T as opus::Trait>::ContentHash) -> DispatchResult {
        let new_like = Like {
            from: sender.clone(),
            to: to,
        };

        let like_id = <NextLikeId<T>>::get();

        let opus_liked_count = Self::opus_liked_count(to);
        let new_opus_liked_count = opus_liked_count.checked_add(1)
            .ok_or("Exceed opus max likes count")?;

        <Likes<T>>::insert(like_id, new_like);
        <NextLikeId<T>>::mutate(|id| *id += <T::LikeId as One>::one());
        <OpusLikedCount<T>>::insert(to, new_opus_liked_count);
        <LikedOpus<T>>::insert((sender.clone(), to), like_id);

        Self::deposit_event(RawEvent::Liked(to, sender, like_id, new_opus_liked_count));

        Ok(())
    }
    pub fn do_collect(sender: T::AccountId, opus_owner: T::AccountId,
                     to: <T as opus::Trait>::ContentHash) -> DispatchResult {
        // new collect
        let new_collect = Collect {
            from: sender.clone(),
            to: to,
        };

        let collect_id = <NextCollectId<T>>::get();

        let opus_collected_count = Self::opus_collected_count(to);
        let new_opus_collected_count = opus_collected_count.checked_add(1)
            .ok_or("Exceed opus max collects count")?;

        <Collects<T>>::insert(collect_id, new_collect);
        <NextCollectId<T>>::mutate(|id| *id += <T::CollectId as One>::one());
        <OpusCollectedCount<T>>::insert(to, new_opus_collected_count);
        <CollectedOpus<T>>::insert((sender.clone(), to), collect_id);

        Self::deposit_event(RawEvent::Collected(to, sender, opus_owner, collect_id, new_opus_collected_count));

        Ok(())
    }

    pub fn do_grant(sender: T::AccountId, opus_owner: T::AccountId,
                    to: <T as opus::Trait>::ContentHash, amount: BalanceOf<T>) -> DispatchResult {
        // new grant
        let new_grant = Grant {
            from: sender.clone(),
            to: to,
            amount: amount,
        };

        let grant_id = <NextGrantId<T>>::get();

        let opus_granted_balance = Self::opus_granted_balance(to);
        let new_opus_granted_balance = opus_granted_balance.checked_add(&amount)
            .ok_or("Exceed opus max grants count")?;

        <Grants<T>>::insert(grant_id, new_grant);
        <NextGrantId<T>>::mutate(|id| *id += <T::GrantId as One>::one());
        <OpusGrantedBalance<T>>::insert(to, new_opus_granted_balance);
        <GrantedOpus<T>>::insert((sender.clone(), to), grant_id);
        // TODO: transfer SKY to opus owner
        let free_balance = T::Currency::free_balance(&sender);
        ensure!(free_balance >= amount, "Currency not enough for Grant.");
        T::Currency::transfer(&sender, &opus_owner, amount, ExistenceRequirement::AllowDeath)?;

        Self::deposit_event(RawEvent::Granted(to, sender, opus_owner, amount, grant_id, new_opus_granted_balance));
        Ok(())
    }

    pub fn do_report(sender: T::AccountId, target: <T as opus::Trait>::ContentHash, reason: <T as opus::Trait>::ContentHash) -> DispatchResult {
        // new report
        let new_report = Report {
            from: sender.clone(),
            target,
            reason,
        };

        let report_id = <NextReportId<T>>::get();

        // TODO: lock SKY for the reporter

        <Reports<T>>::insert(report_id, new_report);
        <NextReportId<T>>::mutate(|id| *id += <T::ReportId as One>::one());
        <OpusReportedBy<T>>::insert(target, sender.clone());
        <ReportedOpus<T>>::insert((sender.clone(), target), report_id);

        Self::deposit_event(RawEvent::Reported(target, sender, reason, report_id));

        Ok(())
    }
}
