use alloy::sol;

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    TicketManager,
    "src/contract/abis/TicketManager.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    EventManager,
    "src/contract/abis/EventManager.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    TokenSwap,
    "src/contract/abis/TokenSwap.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    Marketplace,
    "src/contract/abis/Marketplace.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    DIDRegistry,
    "src/contract/abis/DIDRegistry.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    ShowManager,
    "src/contract/abis/ShowManager.json"
}
