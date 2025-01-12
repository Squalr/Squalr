//! collection_binding.rs - defines `CreateViewCollectionBindingInput` and its logic.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Path, Token, Type,
};

/// Representation of the syntax we expect to parse for our macro:
///
/// create_view_model_collection!(
///     view_binding -> MainWindowView,
///     ViewModelBindings -> { set_processes, get_processes }
///     Converter,
///     Comparer,
/// );
///
/// The fields below store each piece of that invocation.
pub struct CreateViewCollectionBindingInput {
    pub view_binding_ident: Ident,
    pub view_type: Path,
    pub bindings_type: Path,
    pub setter_ident: Ident,
    pub getter_ident: Ident,
    pub converter_type: Type,
    pub comparer_type: Type,
}

impl Parse for CreateViewCollectionBindingInput {
    fn parse(input: ParseStream) -> Result<Self> {
        //
        // Example syntax to parse:
        //
        //   view_binding -> MainWindowView,
        //   ViewModelBindings -> { set_processes, get_processes },
        //   Converter,
        //   Comparer,
        //
        // So we parse each piece in turn.

        // 1) `view_binding -> MainWindowView,`
        let view_binding_ident: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let view_type: Path = input.parse()?;
        input.parse::<Token![,]>()?;

        // 2) `ViewModelBindings -> { set_processes, get_processes },`
        let bindings_type: Path = input.parse()?;
        input.parse::<Token![->]>()?;

        let braced_content;
        braced!(braced_content in input); // parse the `{ ... }` block
        let setter_ident: Ident = braced_content.parse()?;
        braced_content.parse::<Token![,]>()?;
        let getter_ident: Ident = braced_content.parse()?;
        input.parse::<Token![,]>()?;

        // 3) `ProcessInfoConverter,`
        let converter_type: Type = input.parse()?;
        input.parse::<Token![,]>()?;

        // 4) `ProcessInfoComparer,` (optional trailing comma)
        let comparer_type: Type = input.parse()?;
        let _ = input.parse::<Token![,]>(); // optional trailing comma

        Ok(Self {
            view_binding_ident,
            view_type,
            bindings_type,
            setter_ident,
            getter_ident,
            converter_type,
            comparer_type,
        })
    }
}

impl CreateViewCollectionBindingInput {
    /// This method returns the generated `TokenStream` that implements
    /// our macro’s expansion.
    pub fn expand(&self) -> TokenStream {
        let Self {
            view_binding_ident,
            view_type,
            bindings_type,
            setter_ident,
            getter_ident,
            converter_type,
            comparer_type,
        } = self;

        // Generate the desired code:
        //
        // 1) Acquire the Weak<MainWindowView> (or default if lock is poisoned).
        // 2) Build the `ViewCollectionBinding::new(...)`.
        quote! {
            {
                // Acquire the Weak<MainWindowView> (or default if lock is poisoned).
                let view_handle = #view_binding_ident
                    .get_view_handle()
                    .lock()
                    .map_or_else(
                        |_| ::slint::Weak::default(),
                        |guard| guard.clone()
                    );

                // Use the absolute path so the user doesn't have to import it.
                ::slint_mvvm::view_collection_binding::ViewCollectionBinding::new(
                    &view_handle,
                    |view: &#view_type, model| {
                        view.global::<#bindings_type>()
                            .#setter_ident(model)
                    },
                    |view: &#view_type| {
                        view.global::<#bindings_type>()
                            .#getter_ident()
                    },
                    |source_item| {
                        use slint_mvvm::view_data_converter::ViewDataConverter;
                        #converter_type::convert_to_view_data(&source_item)
                    },
                    |a, b| {
                        use slint_mvvm::view_data_comparer::ViewDataComparer;
                        #comparer_type::compare(a, b)
                    },
                )
            }
        }
    }
}
