use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn domain_main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    let panic = panic_impl();
    quote! (
        #[global_allocator]
        static HEAP_ALLOCATOR: malloc::HeapAllocator =  malloc::HeapAllocator::new(corelib::alloc_raw_pages);
        #[unsafe(no_mangle)]
        #item
        #panic
    )
    .into()
}

fn panic_impl() -> TokenStream {
    quote!(
        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            let msg = alloc::format!("\x1b[31m{:?}\x1b[0m\n", info);
            let _ = basic::try_write_console(&msg);
            if let Some(domain_id) = shared_heap::try_domain_id() {
                let _ = basic::try_backtrace(domain_id);
            }
            #[cfg(feature = "rust-unwind")]
            {
                if basic::is_initialized() && shared_heap::is_initialized() {
                    basic::unwind_from_panic();
                }
            }
            loop {}
        }
    )
    .into()
}
