pub(crate) fn get_tcs_num() -> usize {
    if sgx_trts::enclave::rsgx_is_supported_EDMM() {
        sgx_trts::enclave::SgxGlobalData::new().get_dyn_tcs_num() as usize
    } else {
        (sgx_trts::enclave::SgxGlobalData::new().get_tcs_max_num() - 1) as usize
    }
}
