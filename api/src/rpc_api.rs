use std::net::IpAddr;
use std::net::SocketAddr;

use neptune_cash::application::rpc::auth as rpc_auth;
use neptune_cash::application::rpc::server::RpcResult;
use neptune_types::address::KeyType;
use neptune_types::address::ReceivingAddress;
use neptune_types::address::SpendingKey;
use neptune_types::announcement::Announcement;
use neptune_types::block_height::BlockHeight;
use neptune_types::block_info::BlockInfo;
use neptune_types::block_selector::BlockSelector;
use neptune_types::dashboard_overview_data_from_client::DashBoardOverviewDataFromClient;
use neptune_types::mempool_transaction_info::MempoolTransactionInfo;
use neptune_types::native_currency_amount::NativeCurrencyAmount;
use neptune_types::network::Network;
use neptune_types::peer_info::PeerInfo;
use neptune_types::timestamp::Timestamp;
use neptune_types::transaction_kernel::TransactionKernel;
use neptune_types::transaction_kernel_id::TransactionKernelId;
use twenty_first::prelude::*;

#[tarpc::service]
pub trait RPC {
    /// Returns a [rpc_auth::CookieHint] for purposes of zero-conf authentication
    async fn cookie_hint() -> RpcResult<rpc_auth::CookieHint>;

    /// Return the network this neptune-core instance is running
    async fn network() -> RpcResult<Network>;

    /// Returns local socket used for incoming peer-connections. Does not show
    async fn own_listen_address_for_peers(token: rpc_auth::Token) -> RpcResult<Option<SocketAddr>>;

    // /// Return the node's instance-ID which is a globally unique random generated number
    // async fn own_instance_id(token: rpc_auth::Token) -> RpcResult<InstanceId>;

    /// Returns the current block height.
    async fn block_height(token: rpc_auth::Token) -> RpcResult<BlockHeight>;

    /// Returns the number of blocks (confirmations) since wallet balance last changed.
    async fn confirmations(token: rpc_auth::Token) -> RpcResult<Option<BlockHeight>>;

    /// Returns info about the peers we are connected to
    async fn peer_info(token: rpc_auth::Token) -> RpcResult<Vec<PeerInfo>>;

    /// Returns the digest of the latest n blocks
    async fn latest_tip_digests(token: rpc_auth::Token, n: usize) -> RpcResult<Vec<Digest>>;

    /// Returns information about the specified block if found
    async fn block_info(
        token: rpc_auth::Token,
        block_selector: BlockSelector,
    ) -> RpcResult<Option<BlockInfo>>;

    // /// Return the block kernel if block is known.
    // async fn block_kernel(
    //     token: rpc_auth::Token,
    //     block_selector: BlockSelector,
    // ) -> RpcResult<Option<BlockKernel>>;

    /// Return the announements contained in a specified block.
    async fn announcements_in_block(
        token: rpc_auth::Token,
        block_selector: BlockSelector,
    ) -> RpcResult<Option<Vec<Announcement>>>;

    /// Return the digests of known blocks with specified height.
    async fn block_digests_by_height(
        token: rpc_auth::Token,
        height: BlockHeight,
    ) -> RpcResult<Vec<Digest>>;

    /// Return the digest for the specified block if found
    async fn block_digest(
        token: rpc_auth::Token,
        block_selector: BlockSelector,
    ) -> RpcResult<Option<Digest>>;

    /// Return the digest for the specified UTXO leaf index if found
    async fn utxo_digest(token: rpc_auth::Token, leaf_index: u64) -> RpcResult<Option<Digest>>;

    // /// Returns the block digest in which the specified UTXO was created, if available
    // async fn utxo_origin_block(
    //     token: rpc_auth::Token,
    //     addition_record: AdditionRecord,
    //     max_search_depth: Option<u64>,
    // ) -> RpcResult<Option<Digest>>;

    // /// Return the block header for the specified block
    // async fn header(
    //     token: rpc_auth::Token,
    //     block_selector: BlockSelector,
    // ) -> RpcResult<Option<BlockHeader>>;

    /// Get sum of confirmed, unspent, available UTXOs
    async fn confirmed_available_balance(token: rpc_auth::Token)
        -> RpcResult<NativeCurrencyAmount>;

    /// Get sum of unconfirmed, unspent available UTXOs
    async fn unconfirmed_available_balance(
        token: rpc_auth::Token,
    ) -> RpcResult<NativeCurrencyAmount>;

    /// Get the client's wallet transaction history
    async fn history(
        token: rpc_auth::Token,
    ) -> RpcResult<Vec<(Digest, BlockHeight, Timestamp, NativeCurrencyAmount)>>;

    // /// Return information about funds in the wallet
    // async fn wallet_status(token: rpc_auth::Token) -> RpcResult<WalletStatus>;

    /// Return the number of expected UTXOs, including already received UTXOs.
    async fn num_expected_utxos(token: rpc_auth::Token) -> RpcResult<u64>;

    /// generate a new receiving address of the specified type
    async fn next_receiving_address(
        token: rpc_auth::Token,
        key_type: KeyType,
    ) -> RpcResult<ReceivingAddress>;

    /// Return all known keys, for every [KeyType]
    async fn known_keys(token: rpc_auth::Token) -> RpcResult<Vec<SpendingKey>>;

    /// Return known keys for the provided [KeyType]
    async fn known_keys_by_keytype(
        token: rpc_auth::Token,
        key_type: KeyType,
    ) -> RpcResult<Vec<SpendingKey>>;

    /// Return the number of transactions in the mempool
    async fn mempool_tx_count(token: rpc_auth::Token) -> RpcResult<usize>;

    async fn mempool_size(token: rpc_auth::Token) -> RpcResult<usize>;

    /// Return info about the transactions in the mempool
    async fn mempool_overview(
        token: rpc_auth::Token,
        start_index: usize,
        number: usize,
    ) -> RpcResult<Vec<MempoolTransactionInfo>>;

    /// Return transaction kernel by id if found in mempool.
    async fn mempool_tx_kernel(
        token: rpc_auth::Token,
        tx_kernel_id: TransactionKernelId,
    ) -> RpcResult<Option<TransactionKernel>>;

    /// Return the information used on the dashboard's overview tab
    async fn dashboard_overview_data(
        token: rpc_auth::Token,
    ) -> RpcResult<DashBoardOverviewDataFromClient>;

    /// Determine whether the user-supplied string is a valid address
    async fn validate_address(
        token: rpc_auth::Token,
        address: String,
        network: Network,
    ) -> RpcResult<Option<ReceivingAddress>>;

    /// Determine whether the user-supplied string is a valid amount
    async fn validate_amount(
        token: rpc_auth::Token,
        amount: String,
    ) -> RpcResult<Option<NativeCurrencyAmount>>;

    /// Determine whether the given amount is less than (or equal to) the balance
    async fn amount_leq_confirmed_available_balance(
        token: rpc_auth::Token,
        amount: NativeCurrencyAmount,
    ) -> RpcResult<bool>;

    // /// Generate a report of all owned and unspent coins, whether time-locked or not.
    // async fn list_own_coins(token: rpc_auth::Token) -> RpcResult<Vec<CoinWithPossibleTimeLock>>;

    /// Get CPU temperature.
    async fn cpu_temp(token: rpc_auth::Token) -> RpcResult<Option<f32>>;

    // /// Get the proof-of-work puzzle for the current block proposal. Uses the
    // async fn pow_puzzle_internal_key(
    //     token: rpc_auth::Token,
    // ) -> RpcResult<Option<ProofOfWorkPuzzle>>;

    // /// Get the proof-of-work puzzle for the current block proposal. Like
    // async fn pow_puzzle_external_key(
    //     token: rpc_auth::Token,
    //     guesser_digest: Digest,
    // ) -> RpcResult<Option<ProofOfWorkPuzzle>>;

    /// Return the block intervals of a range of blocks. Return value is the
    async fn block_intervals(
        token: rpc_auth::Token,
        last_block: BlockSelector,
        max_num_blocks: Option<usize>,
    ) -> RpcResult<Option<Vec<(u64, u64)>>>;

    // /// Return the difficulties of a range of blocks.
    // async fn block_difficulties(
    //     token: rpc_auth::Token,
    //     last_block: BlockSelector,
    //     max_num_blocks: Option<usize>,
    // ) -> RpcResult<Vec<(u64, Difficulty)>>;

    /// Broadcast transaction notifications for all transactions in this node's
    async fn broadcast_all_mempool_txs(token: rpc_auth::Token) -> RpcResult<()>;

    /// Clears standing for all peers, connected or not
    async fn clear_all_standings(token: rpc_auth::Token) -> RpcResult<()>;

    /// Clears standing for ip, whether connected or not
    async fn clear_standing_by_ip(token: rpc_auth::Token, ip: IpAddr) -> RpcResult<()>;

    // /// todo: docs.
    // async fn spendable_inputs(token: rpc_auth::Token) -> RpcResult<TxInputList>;

    // /// retrieve spendable inputs sufficient to cover spend_amount by applying selection policy.
    // async fn select_spendable_inputs(
    //     token: rpc_auth::Token,
    //     policy: InputSelectionPolicy,
    //     spend_amount: NativeCurrencyAmount,
    // ) -> RpcResult<TxInputList>;

    // /// generate tx outputs from list of OutputFormat.
    // async fn generate_tx_outputs(
    //     token: rpc_auth::Token,
    //     outputs: Vec<OutputFormat>,
    // ) -> RpcResult<TxOutputList>;

    // /// todo: docs.
    // async fn generate_tx_details(
    //     token: rpc_auth::Token,
    //     tx_inputs: TxInputList,
    //     tx_outputs: TxOutputList,
    //     change_policy: ChangePolicy,
    //     fee: NativeCurrencyAmount,
    // ) -> RpcResult<TransactionDetails>;

    // /// todo: docs.
    // async fn generate_witness_proof(
    //     token: rpc_auth::Token,
    //     tx_details: TransactionDetails,
    // ) -> RpcResult<TransactionProof>;

    // /// assemble a transaction from TransactionDetails and a TransactionProof.
    // async fn assemble_transaction(
    //     token: rpc_auth::Token,
    //     transaction_details: TransactionDetails,
    //     transaction_proof: TransactionProof,
    // ) -> RpcResult<Transaction>;

    // /// assemble transaction artifacts from TransactionDetails and a TransactionProof.
    // async fn assemble_transaction_artifacts(
    //     token: rpc_auth::Token,
    //     transaction_details: TransactionDetails,
    //     transaction_proof: TransactionProof,
    // ) -> RpcResult<TxCreationArtifacts>;

    // /// record transaction and initiate broadcast to peers
    // async fn record_and_broadcast_transaction(
    //     token: rpc_auth::Token,
    //     tx_artifacts: TxCreationArtifacts,
    // ) -> RpcResult<()>;

    // /// Send coins to one or more recipients
    // async fn send(
    //     token: rpc_auth::Token,
    //     outputs: Vec<OutputFormat>,
    //     change_policy: ChangePolicy,
    //     fee: NativeCurrencyAmount,
    // ) -> RpcResult<TxCreationArtifacts>;

    // /// upgrades a transaction's proof.
    // async fn upgrade_tx_proof(
    //     token: rpc_auth::Token,
    //     transaction_id: TransactionKernelId,
    //     transaction_proof: TransactionProof,
    // ) -> RpcResult<()>;

    // /// todo: docs.
    // async fn proof_type(
    //     token: rpc_auth::Token,
    //     txid: TransactionKernelId,
    // ) -> RpcResult<TransactionProofType>;

    /// claim a utxo
    async fn claim_utxo(
        token: rpc_auth::Token,
        utxo_transfer_encrypted: String,
        max_search_depth: Option<u64>,
    ) -> RpcResult<bool>;

    /// Delete all transactions from the mempool.
    async fn clear_mempool(token: rpc_auth::Token) -> RpcResult<()>;

    /// Stop miner if running
    async fn pause_miner(token: rpc_auth::Token) -> RpcResult<()>;

    /// Start miner if not running
    async fn restart_miner(token: rpc_auth::Token) -> RpcResult<()>;

    /// mine a series of blocks to the node's wallet.
    async fn mine_blocks_to_wallet(token: rpc_auth::Token, n_blocks: u32) -> RpcResult<()>;

    /// Provide a PoW-solution to the current block proposal.
    async fn provide_pow_solution(
        token: rpc_auth::Token,
        nonce: Digest,
        proposal_id: Digest,
    ) -> RpcResult<bool>;

    /// mark MUTXOs as abandoned
    async fn prune_abandoned_monitored_utxos(token: rpc_auth::Token) -> RpcResult<usize>;

    /// Gracious shutdown.
    async fn shutdown(token: rpc_auth::Token) -> RpcResult<bool>;
}
