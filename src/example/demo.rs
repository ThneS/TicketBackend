#[tokio::main]
async fn main() -> Result<()> {
    let ws_url = "ws://127.0.0.1:8545";
    let ws = WsConnect::new(ws_url);
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse()?;
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_ws(ws)
        .await
        .unwrap();
    let tm_instance = get_ticket_manager_instance(provider.clone())?;

    let owner = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
    let tx_handler = tm_instance
        .setMinterAuthorization(owner, true)
        .send()
        .await?;
    // 获取交易回执
    let receipt = tx_handler.get_receipt().await?;

    for log in receipt.logs() {
        if log.address() != *tm_instance.address() {
            continue;
        }
        // 安全地读取第一个 topic 并与事件签名比对
        if let Some(topic0) = log.topics().first() {
            if *topic0 == keccak256("MinterAuthorized(address,bool)") {
                println!("MinterAuthorized event found: {:?}", log);

                // 从 indexed topic[1] 解码出 minter（address 占低 20 字节）
                if let Some(minter_topic) = log.topics().get(1) {
                    let bytes = minter_topic.as_slice();
                    let minter = alloy::primitives::Address::from_slice(&bytes[12..]);
                    println!("Minter: {:?}", minter);
                }
            }
        }
    }
    let address = tm_instance.address();
    println!("TicketManager address: {:?}", address);
    println!("Hello, world!");
    Ok(())
}
