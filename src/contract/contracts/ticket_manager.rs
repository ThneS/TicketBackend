use crate::contract::{
    AddressMap, bindings::TicketManager::TicketManagerInstance,
};
use alloy::{primitives::Address, providers::Provider};
use eyre::Result;

pub fn get_ticket_manager_instance_with_address<P: Provider>(
    provider: P,
    address: Address,
) -> Result<TicketManagerInstance<P>> {
    Ok(TicketManagerInstance::new(address, provider))
}

pub fn get_ticket_manager_instance_from_map<P: Provider>(
    provider: P,
    addresses: &AddressMap,
) -> Result<TicketManagerInstance<P>> {
    get_ticket_manager_instance_with_address(provider, addresses.show_manager)
}
