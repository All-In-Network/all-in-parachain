use allin::runtime_types;
use futures::StreamExt;
use sp_keyring::{ed25519::ed25519::Pair, AccountKeyring};
use subxt::{
    config::WithExtrinsicParams,
    ext::sp_runtime::AccountId32,
    tx::{BaseExtrinsicParams, PairSigner, PlainTip},
    OnlineClient, PolkadotConfig, SubstrateConfig,
};
type Call = runtime_types::all_in_runtime::Call;
type FundingCall = runtime_types::pallet_funding::funding_trador::pallet::Call;
type ApiClient = OnlineClient<
    WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

#[subxt::subxt(runtime_metadata_path = "./metadata/metadata.scale")]
pub mod allin {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::new().await?;

    let signer = AccountKeyring::Alice;
    //let dest = AccountKeyring::Bob.to_account_id().into();
    let _ = set_funding_manager(api.clone(), signer.clone()).await?;

    let proxy = create_new_funding_account(api.clone(), signer.clone()).await?;

    println!("proxy: {}", proxy.unwrap());

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

async fn create_new_funding_account(
    api: ApiClient,
    signer: AccountKeyring,
) -> Result<Option<AccountId32>, Box<dyn std::error::Error>> {
    let manager = PairSigner::new(signer.pair());
    let create_new_tx = allin::tx().funding().create_new_funding_account(
        runtime_types::all_in_runtime::ProxyType::Any,
        0,
        0,
    );

    let create_new_tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&create_new_tx, &manager)
        .await?
        .wait_for_finalized_success()
        .await?;

    let tx_event = create_new_tx_progress.find_first::<allin::proxy::events::AnonymousCreated>()?;

    if let Some(event) = tx_event {
        println!("New proxy cerated: {event:#?}");
        return Ok(Some(event.anonymous.clone()));
    } else {
        println!("Failed to find Event");
    }
    Ok(None)
}
