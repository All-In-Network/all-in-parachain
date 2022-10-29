use allin::runtime_types::{
    self, pallet_marketplace_nfts::types::StatusType, sp_runtime::bounded::bounded_vec::BoundedVec,
};
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use subxt::{
    config::WithExtrinsicParams,
    ext::sp_runtime::{AccountId32, MultiAddress},
    tx::{BaseExtrinsicParams, PairSigner, PlainTip},
    OnlineClient, PolkadotConfig, SubstrateConfig,
};

type Call = runtime_types::all_in_runtime::Call;
type BalancesCall = runtime_types::pallet_balances::pallet::Call;
type FundingCall = runtime_types::pallet_funding::funding_trador::pallet::Call;
type NftSaleCall = runtime_types::pallet_marketplace_nfts::nft_sale::pallet::Call;

type ApiClient = OnlineClient<
    WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

#[subxt::subxt(runtime_metadata_path = "./metadata/metadata_node.scale")]
pub mod allin {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::new().await?;
    let signer = AccountKeyring::Alice;

    set_funding_manager(api.clone(), signer.clone()).await?;
    set_nftsale_manager(api.clone(), signer.clone()).await?;

    let treasury_account =
        AccountId32::from_str("5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z")?;

    let treasury_account = MultiAddress::from(treasury_account);

    let metadata = BoundedVec(
        "ipfs://QmfWBWsdtQErNxMHb2UfWBQHTwbTEdCUMetpwR4y9uDNp3"
            .as_bytes()
            .to_vec(),
    );
    let metadata1 = BoundedVec(
        "ipfs://QmfWBWsdtQErNxMHb2UfWBQHTwbTEdCUMetpwR4y9uDNp3"
            .as_bytes()
            .to_vec(),
    );

    let symbol = BoundedVec("Soulbound".as_bytes().to_vec());

    let txs_calls = vec![
        Call::Balances(BalancesCall::transfer {
            dest: treasury_account,
            value: 1_000_000_000_000_000_000,
        }),
        Call::NftSale(NftSaleCall::trador_create_collection {
            metadata: metadata,
            max: None,
            symbol,
        }),
        Call::NftSale(NftSaleCall::set_soulbound_metadata {
            soulbound_metadata: metadata1,
        }),
        Call::NftSale(NftSaleCall::set_soulbound_collection_id { collection_id: 0 }),
        Call::NftSale(NftSaleCall::set_status_type {
            status: true,
            status_type: StatusType::ClaimSoulbound,
        }),
    ];

    let txs = allin::tx().utility().batch(txs_calls);

    let txs_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&txs, &PairSigner::new(signer.pair()))
        .await?
        .wait_for_finalized_success()
        .await?;

    let tx_event = txs_progress.find_first::<allin::utility::events::BatchCompleted>()?;

    if let Some(event) = tx_event {
        println!("Transfer of funds to the treasury successfully completed.");
        println!("NFT Soulbound's collection successfully created.");
        println!("{event:#?}");
    } else {
        println!("Failed to find Event");
    }
    Ok(())
}


async fn set_funding_manager(
    api: ApiClient,
    signer: AccountKeyring,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice = PairSigner::new(signer.pair());
    let call = Call::Funding(FundingCall::set_manager {
        new_manager: signer.to_account_id(),
    });
    let sudo_call_tx = allin::tx().sudo().sudo(call);

    let sudo_call_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&sudo_call_tx, &alice)
        .await?
        .wait_for_finalized_success()
        .await?;

    let tx_event = sudo_call_progress.find_first::<allin::funding::events::ManagerChanged>()?;

    if let Some(event) = tx_event {
        println!("Funding manager created successfully: {event:#?}");
    } else {
        println!("Failed to find Event");
    }

    Ok(())
}


async fn set_nftsale_manager(
    api: ApiClient,
    signer: AccountKeyring,
) -> Result<(), Box<dyn std::error::Error>> {
    let alice = PairSigner::new(signer.pair());
    let call = Call::NftSale(NftSaleCall::set_manager {
        new_manager: signer.to_account_id(),
    });
    let sudo_call_tx = allin::tx().sudo().sudo(call);

    let sudo_call_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&sudo_call_tx, &alice)
        .await?
        .wait_for_finalized_success()
        .await?;

    let tx_event = sudo_call_progress.find_first::<allin::nft_sale::events::ManagerChanged>()?;

    if let Some(event) = tx_event {
        println!("NFT Sale manager created successfully: {event:#?}");
    } else {
        println!("Failed to find Event");
    }

    Ok(())
}
