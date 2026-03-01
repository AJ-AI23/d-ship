#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    tracker
    (
        init => init
        upgrade => upgrade
        getConfig => config
        register_event => register_event
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
