use alloy::sol;

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    TicketManager,
    "src/abis/TicketManager.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    EventManager,
    "src/abis/EventManager.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    TokenSwap,
    "src/abis/TokenSwap.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    Marketplace,
    "src/abis/Marketplace.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    DIDRegistry,
    "src/abis/DIDRegistry.json"
}

sol! {
    #[sol(rpc)]
    #[derive(Debug)]
    ShowManager,
    "src/abis/ShowManager.json"
}
