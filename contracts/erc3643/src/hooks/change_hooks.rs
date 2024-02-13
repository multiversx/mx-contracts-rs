use super::hook_type::{Hook, ErcHookType};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ChangeHooksModule: super::call_hook::CallHookModule {
    #[only_owner]
    #[endpoint(addHook)]
    fn add_hook(&self, hook_type: ErcHookType, to: ManagedAddress, endpoint_name: ManagedBuffer) {
        self.require_sc_address(&to);
        self.require_not_empty_buffer(&endpoint_name);

        self.hooks(hook_type).update(|hooks| {
            hooks.push(Hook {
                dest_address: to,
                endpoint_name,
            })
        });
    }

    #[only_owner]
    #[endpoint(removeHook)]
    fn remove_hook(
        &self,
        hook_type: ErcHookType,
        to: ManagedAddress,
        endpoint_name: ManagedBuffer,
    ) {
        self.hooks(hook_type).update(|hooks| {
            let opt_index = hooks.find(&Hook {
                dest_address: to,
                endpoint_name,
            });

            require!(opt_index.is_some(), "Item not found");

            let index = unsafe { opt_index.unwrap_unchecked() };
            hooks.remove(index);
        });
    }

    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }

    fn require_not_empty_buffer(&self, buffer: &ManagedBuffer) {
        require!(!buffer.is_empty(), "Empty buffer");
    }
}
