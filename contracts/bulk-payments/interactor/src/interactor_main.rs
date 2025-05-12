
use multiversx_sc_snippets::imports::*;
use bulk_payments_interact::bulk_payments_cli;

#[tokio::main]
async fn main() {
    bulk_payments_cli().await;
}  

