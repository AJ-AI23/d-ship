#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    parcel
    (
        init => init
        upgrade => upgrade
        getConfig => config
        register_parcel => register_parcel
        getConfigHash => get_config_hash
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
