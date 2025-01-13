mod collection_binding;
mod view_bindings;

use crate::collection_binding::CreateViewCollectionBindingInput;
use crate::view_bindings::CreateViewBindingsInput;
use proc_macro::TokenStream;
use syn::parse_macro_input;

// -------------------------------------------------------------------------------------
// Top-level macro entries
// -------------------------------------------------------------------------------------
#[proc_macro]
pub fn create_view_bindings(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as CreateViewBindingsInput);
    parsed.expand().into()
}

#[proc_macro]
pub fn create_view_model_collection(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as CreateViewCollectionBindingInput);
    parsed.expand().into()
}
