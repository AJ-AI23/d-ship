#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    serial
    (
        init => init
        upgrade => upgrade
        getConfig => config
        generate => generate
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
