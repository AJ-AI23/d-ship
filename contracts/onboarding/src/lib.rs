//! Onboarding contract: deploys Agreement contracts after verifying customer signatures.
//!
//! Verifies Ed25519 signature over approval message, checks expiry and nonce, then deploys
//! from the Agreement template.
#![no_std]

use dship_common::agreement;
use multiversx_sc::{
    imports::*,
    types::CodeMetadata,
};

const DEPLOY_GAS: u64 = 100_000_000;

#[multiversx_sc::contract]
pub trait Onboarding {
    #[view(getAgreementTemplate)]
    #[storage_mapper("agreement_template")]
    fn agreement_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getAllowedCodeHash)]
    #[storage_mapper("allowed_code_hash")]
    fn allowed_code_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getCarrierAddress)]
    #[storage_mapper("carrier_address")]
    fn carrier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("deployed_accounts")]
    fn deployed_accounts(&self) -> SetMapper<ManagedAddress>;

    #[storage_mapper("used_nonce")]
    fn used_nonce(&self, customer: &ManagedAddress, nonce: &u64) -> SingleValueMapper<u8>;

    #[view(getDeployedAccounts)]
    fn get_deployed_accounts(&self) -> MultiValueEncoded<ManagedAddress> {
        let mut result = MultiValueEncoded::new();
        for addr in self.deployed_accounts().iter() {
            result.push(addr);
        }
        result
    }

    #[init]
    fn init(
        &self,
        agreement_template: ManagedAddress,
        allowed_code_hash: ManagedBuffer,
    ) {
        self.agreement_template().set(&agreement_template);
        self.allowed_code_hash().set(&allowed_code_hash);
        self.carrier_address().set(&self.blockchain().get_caller());
    }

    /// Deploy an Agreement for a customer who has signed the approval message.
    #[endpoint(deployAgreement)]
    #[allow_multiple_var_args]
    fn deploy_agreement(
        &self,
        customer_owner: ManagedAddress,
        customer_pubkey: ManagedBuffer,
        agreement_config_hash: ManagedBuffer,
        shipment_contract: ManagedAddress,
        expiry: u64,
        nonce: u64,
        credit_limit: BigUint,
        enabled_services: MultiValueEncoded<ManagedBuffer>,
        signature: ManagedBuffer,
    ) -> ManagedAddress {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.carrier_address().get(),
            "Only carrier may deploy"
        );
        require!(
            self.blockchain().get_block_timestamp() <= expiry,
            "Approval expired"
        );
        require!(
            self.used_nonce(&customer_owner, &nonce).is_empty(),
            "Nonce already used"
        );

        let factory_addr = self.blockchain().get_sc_address();
        let carrier_addr = self.carrier_address().get();
        let template = self.agreement_template().get();
        let code_hash = self.allowed_code_hash().get();

        let msg = agreement::build_approval_message(
            &carrier_addr,
            &factory_addr,
            &code_hash,
            &agreement_config_hash,
            expiry,
            nonce,
        );
        let msg_hash = self.crypto().keccak256(&msg);
        let msg_hash_buf = ManagedBuffer::from(msg_hash.to_byte_array().as_slice());
        self.crypto().verify_ed25519(&customer_pubkey, &msg_hash_buf, &signature);

        let mut arg_buffer = ManagedArgBuffer::new();
        arg_buffer.push_arg(customer_owner.clone());
        arg_buffer.push_arg(carrier_addr.clone());
        arg_buffer.push_arg(shipment_contract.clone());
        arg_buffer.push_arg(agreement_config_hash.clone());
        arg_buffer.push_arg(credit_limit);
        for s in enabled_services {
            arg_buffer.push_arg(s);
        }

        let code_metadata = CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;
        let (new_address, _) = self.send_raw().deploy_from_source_contract(
            DEPLOY_GAS,
            &BigUint::zero(),
            &template,
            code_metadata,
            &arg_buffer,
        );

        self.used_nonce(&customer_owner, &nonce).set(1u8);
        self.deployed_accounts().insert(new_address.clone());

        new_address
    }
}
