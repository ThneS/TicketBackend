use crate::bindings::TicketManager::TicketManagerInstance;
use alloy::primitives::address;
use alloy::providers::Provider;
use eyre::Result;

pub fn get_ticket_manager_instance<P: Provider>(provider: P) -> Result<TicketManagerInstance<P>> {
    let address = address!("0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512");
    Ok(TicketManagerInstance::new(address, provider))
}
