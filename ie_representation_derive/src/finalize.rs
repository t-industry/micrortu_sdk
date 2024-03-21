use micrortu_build_utils::WasmBlobDump;
use proc_macro::TokenStream;
use quote::quote;

use crate::{BLOCKS, FINALIZED, STRINGS};

pub fn finalize() -> TokenStream {
    if FINALIZED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        panic!("finalize! can only be called once");
    }
    let final_string = STRINGS.lock().expect("poison").clone();
    let blocks = BLOCKS.lock().expect("poison").clone();
    let len = final_string.len();
    let bytes_array = final_string.as_bytes().iter().map(|&b| quote! { #b });
    let bytes_array = quote! { [ #(#bytes_array),* ] };
    let doc = format!(" Collected strings: {final_string:?}");

    let blocks = blocks.values().cloned().collect();
    let metadata = WasmBlobDump { blocks };

    let metadata = serde_json::to_string(&metadata).expect("serialization error");
    let metadata_len = metadata.len();
    let metadata_bytes_array = metadata.as_bytes().iter().map(|&b| quote! { #b });
    let metadata_bytes_array = quote! { [ #(#metadata_bytes_array),* ] };

    let expanded = quote! {
        #[no_mangle]
        #[doc = #doc]
        static COLLECTED_STRINGS: [u8; #len] = #bytes_array;

        #[link_section = "metadata"]
        #[allow(dead_code)]
        static META: [u8; #metadata_len] = #metadata_bytes_array;
    };

    expanded.into()
}
