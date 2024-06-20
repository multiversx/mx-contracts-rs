multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CallbacksModule {
    /// Callback only performs logging.
    #[callback]
    fn perform_async_call_callback(
        &self,
        #[call_result] call_result: ManagedAsyncCallResult<MultiValueEncoded<ManagedBuffer>>,
    ) {
        match call_result {
            ManagedAsyncCallResult::Ok(results) => {
                self.async_call_success(results);
            }
            ManagedAsyncCallResult::Err(err) => {
                self.async_call_error(err.err_code, err.err_msg);
            }
        }
    }

    #[event("asyncCallSuccess")]
    fn async_call_success(&self, #[indexed] results: MultiValueEncoded<ManagedBuffer>);

    #[event("asyncCallError")]
    fn async_call_error(&self, #[indexed] err_code: u32, #[indexed] err_message: ManagedBuffer);
}
