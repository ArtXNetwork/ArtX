#![warn(missing_docs)]

use std::sync::Arc;

use primitives::{Block, BlockNumber, AccountId, Index, Balance, Hash};
use runtime::UncheckedExtrinsic;
use sp_api::ProvideRuntimeApi;
use sp_transaction_pool::TransactionPool;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sp_consensus::SelectChain;
use sc_keystore::KeyStorePtr;
use sp_consensus_babe::BabeApi;
use sc_consensus_epochs::SharedEpochChanges;
use sc_consensus_babe::{Config, Epoch};
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_finality_grandpa::{SharedVoterState, SharedAuthoritySet};
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
use sc_rpc_api::DenyUnsafe;
use sp_block_builder::BlockBuilder;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Remote access to the blockchain (async).
	pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
	/// Fetcher instance.
	pub fetcher: Arc<F>,
}

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: Config,
	/// BABE pending epoch changes.
	pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: KeyStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps {
	/// Voting round info.
	pub shared_voter_state: SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, M, SC>(
	deps: FullDeps<C, P, SC>,
) -> jsonrpc_core::IoHandler<M> where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UncheckedExtrinsic>,
	C::Api: BabeApi<Block>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
	M: jsonrpc_core::Metadata + Default,
	SC: SelectChain<Block> +'static,
{
	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	use pallet_contracts_rpc::{Contracts, ContractsApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		select_chain,
		deny_unsafe,
		babe,
		grandpa,
	} = deps;
	let BabeDeps {
		keystore,
		babe_config,
		shared_epoch_changes,
	} = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
	} = grandpa;

	io.extend_with(
		SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))
	);
	// Making synchronous calls in light client freezes the browser currently,
	// more context: https://github.com/paritytech/substrate/pull/3480
	// These RPCs should use an asynchronous caller instead.
	io.extend_with(
		ContractsApi::to_delegate(Contracts::new(client.clone()))
	);
	io.extend_with(
		TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
	);
	io.extend_with(
		sc_consensus_babe_rpc::BabeApi::to_delegate(
			BabeRpcHandler::new(
				client,
				shared_epoch_changes,
				keystore,
				babe_config,
				select_chain,
				deny_unsafe,
			),
		)
	);
	io.extend_with(
		sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
			GrandpaRpcHandler::new(shared_authority_set, shared_voter_state)
		)
	);

	io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(
	deps: LightDeps<C, F, P>,
) -> jsonrpc_core::IoHandler<M> where
	C: sp_blockchain::HeaderBackend<Block>,
	C: Send + Sync + 'static,
	F: sc_client_api::light::Fetcher<Block> + 'static,
	P: TransactionPool + 'static,
	M: jsonrpc_core::Metadata + Default,
{
	use substrate_frame_rpc_system::{LightSystem, SystemApi};

	let LightDeps {
		client,
		pool,
		remote_blockchain,
		fetcher
	} = deps;
	let mut io = jsonrpc_core::IoHandler::default();
	io.extend_with(
		SystemApi::<Hash, AccountId, Index>::to_delegate(LightSystem::new(client, remote_blockchain, fetcher, pool))
	);

	io
}




//#![warn(missing_docs)]
//
//use std::{sync::Arc, fmt};
//
//use primitives::{Block, BlockNumber, AccountId, Index, Balance, Hash};
//use runtime::UncheckedExtrinsic;
//use sp_api::ProvideRuntimeApi;
//use sp_transaction_pool::TransactionPool;
//use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
//use sp_consensus::SelectChain;
//use sc_keystore::KeyStorePtr;
//use sp_consensus_babe::BabeApi;
//use sc_consensus_epochs::SharedEpochChanges;
//use sc_consensus_babe::{Config, Epoch};
//use sc_consensus_babe_rpc::BabeRpcHandler;
//use sc_finality_grandpa::{SharedVoterState, SharedAuthoritySet};
//use sc_finality_grandpa_rpc::GrandpaRpcHandler;
//use sc_rpc_api::DenyUnsafe;
//
///// Light client extra dependencies.
//pub struct LightDeps<C, F, P> {
//	/// The client instance to use.
//	pub client: Arc<C>,
//	/// Transaction pool instance.
//	pub pool: Arc<P>,
//	/// Remote access to the blockchain (async).
//	pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
//	/// Fetcher instance.
//	pub fetcher: Arc<F>,
//}
//
///// Extra dependencies for BABE.
//pub struct BabeDeps {
//	/// BABE protocol config.
//	pub babe_config: Config,
//	/// BABE pending epoch changes.
//	pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
//	/// The keystore that manages the keys of the node.
//	pub keystore: KeyStorePtr,
//}
//
///// Extra dependencies for GRANDPA
//pub struct GrandpaDeps {
//	/// Voting round info.
//	pub shared_voter_state: SharedVoterState,
//	/// Authority set info.
//	pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
//}
//
///// Full client dependencies.
//pub struct FullDeps<C, P, SC> {
//	/// The client instance to use.
//	pub client: Arc<C>,
//	/// Transaction pool instance.
//	pub pool: Arc<P>,
//	/// The SelectChain Strategy
//	pub select_chain: SC,
//	/// Whether to deny unsafe calls
//	pub deny_unsafe: DenyUnsafe,
//	/// BABE specific dependencies.
//	pub babe: BabeDeps,
//	/// GRANDPA specific dependencies.
//	pub grandpa: GrandpaDeps,
//}
//
///// Instantiate all Full RPC extensions.
//pub fn create_full<C, P, M, SC>(
//	deps: FullDeps<C, P, SC>,
//) -> jsonrpc_core::IoHandler<M> where
//	C: ProvideRuntimeApi<Block>,
//	C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError> + 'static,
//	C: Send + Sync + 'static,
//	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
//	C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber>,
//	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UncheckedExtrinsic>,
//	C::Api: BabeApi<Block>,
//	<C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
//	P: TransactionPool + 'static,
//	M: jsonrpc_core::Metadata + Default,
//	SC: SelectChain<Block> +'static,
//{
//	use substrate_frame_rpc_system::{FullSystem, SystemApi};
//	use pallet_contracts_rpc::{Contracts, ContractsApi};
//	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
//
//	let mut io = jsonrpc_core::IoHandler::default();
//	let FullDeps {
//		client,
//		pool,
//		select_chain,
//		deny_unsafe,
//		babe,
//		grandpa,
//	} = deps;
//	let BabeDeps {
//		keystore,
//		babe_config,
//		shared_epoch_changes,
//	} = babe;
//	let GrandpaDeps {
//		shared_voter_state,
//		shared_authority_set,
//	} = grandpa;
//
//	io.extend_with(
//		SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))
//	);
//	// Making synchronous calls in light client freezes the browser currently,
//	// more context: https://github.com/paritytech/substrate/pull/3480
//	// These RPCs should use an asynchronous caller instead.
//	io.extend_with(
//		ContractsApi::to_delegate(Contracts::new(client.clone()))
//	);
//	io.extend_with(
//		TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
//	);
//	io.extend_with(
//		sc_consensus_babe_rpc::BabeApi::to_delegate(
//			BabeRpcHandler::new(
//				client,
//				shared_epoch_changes,
//				keystore,
//				babe_config,
//				select_chain,
//				deny_unsafe,
//			),
//		)
//	);
//	io.extend_with(
//		sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
//			GrandpaRpcHandler::new(shared_authority_set, shared_voter_state)
//		)
//	);
//
//	io
//}
//
///// Instantiate all Light RPC extensions.
//pub fn create_light<C, P, M, F>(
//	deps: LightDeps<C, F, P>,
//) -> jsonrpc_core::IoHandler<M> where
//	C: sp_blockchain::HeaderBackend<Block>,
//	C: Send + Sync + 'static,
//	F: sc_client_api::light::Fetcher<Block> + 'static,
//	P: TransactionPool + 'static,
//	M: jsonrpc_core::Metadata + Default,
//{
//	use substrate_frame_rpc_system::{LightSystem, SystemApi};
//
//	let LightDeps {
//		client,
//		pool,
//		remote_blockchain,
//		fetcher
//	} = deps;
//	let mut io = jsonrpc_core::IoHandler::default();
//	io.extend_with(
//		SystemApi::<Hash, AccountId, Index>::to_delegate(LightSystem::new(client, remote_blockchain, fetcher, pool))
//	);
//
//	io
//}
