//! collection_binding.rs - defines `CreateViewCollectionBindingInput` and its logic.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::Comma,
    Ident, Path, Token,
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
    pub converter_type: Path,
    pub converter_fields: Vec<Ident>,
    pub comparer_type: Path,
    pub comparer_fields: Vec<Ident>,
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

        // 1) `view_binding -> View,`
        let view_binding_ident: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let view_type: Path = input.parse()?;
        input.parse::<Token![,]>()?;

        // 2) `ViewModelBindings -> { setter, getter },`
        let bindings_type: Path = input.parse()?;
        input.parse::<Token![->]>()?;
        let braced_content;
        braced!(braced_content in input); // parse the `{ ... }` block
        let setter_ident: Ident = braced_content.parse()?;
        braced_content.parse::<Token![,]>()?;
        let getter_ident: Ident = braced_content.parse()?;
        input.parse::<Token![,]>()?;

        // 3) `ProcessInfoConverter,`
        // parse: Converter -> [some_field],
        let (converter_type, converter_fields) = parse_type_and_injections(input)?;

        // 4) `ProcessInfoComparer,` (optional trailing comma)
        // parse: Comparer -> [maybe_other_field]
        let (comparer_type, comparer_fields) = parse_type_and_injections(input)?;
        // optional trailing comma
        let _ = input.parse::<Option<Comma>>();

        Ok(Self {
            view_binding_ident,
            view_type,
            bindings_type,
            setter_ident,
            getter_ident,
            converter_type,
            converter_fields,
            comparer_type,
            comparer_fields,
        })
    }
}

fn parse_type_and_injections(input: ParseStream) -> Result<(Path, Vec<Ident>)> {
    let path: Path = input.parse()?;
    input.parse::<Token![->]>()?;

    // Parse bracketed fields: e.g. `[field1, field2, ...]`
    let bracketed_content;
    bracketed!(bracketed_content in input);
    let fields: Punctuated<Ident, Token![,]> = bracketed_content.parse_terminated(Ident::parse, Token![,])?;

    input.parse::<Token![,]>()?;

    Ok((path, fields.into_iter().collect()))
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
            converter_fields,
            comparer_type,
            comparer_fields,
        } = self;

        let converter_ident = format_ident!("converter");
        let comparer_ident = format_ident!("comparer");

        let converter_injections = converter_fields.iter().map(|f| quote! { #f.clone() });
        let comparer_injections = comparer_fields.iter().map(|f| quote! { #f.clone() });

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

                let #converter_ident = ::std::sync::Arc::new(#converter_type::new(#(#converter_injections),*));
                let #comparer_ident = ::std::sync::Arc::new(#comparer_type::new(#(#comparer_injections),*));

                // Use the absolute path so the user doesn't have to import it.
                ::slint_mvvm::view_collection_binding::ViewCollectionBinding::new(
                    &view_handle,
                    |view: &#view_type, model| {
                        view.global::<#bindings_type>().#setter_ident(model)
                    },
                    |view: &#view_type| {
                        view.global::<#bindings_type>().#getter_ident()
                    },
                    {
                        // Capture Arc in a move closure
                        let converter = #converter_ident.clone();
                        move |source_collection| {
                            use slint_mvvm::view_data_converter::ViewDataConverter;
                            converter.convert_collection(&source_collection)
                        }
                    },
                    {
                        // Capture Arc in a move closure
                        let comparer = #comparer_ident.clone();
                        move |a, b| {
                            use slint_mvvm::view_data_comparer::ViewDataComparer;
                            comparer.compare(a, b)
                        }
                    },
                )
            }
        }
    }
}
