//
// Cargo.toml for your proc-macro crate must include:
// [lib]
// proc-macro = true
//
// This file provides a `create_view_bindings!` macro that supports multi-binding formats.
// The MainWindowViewModel implementation at the end demonstrates usage.
//

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Path, Result as SynResult, Token,
};

/// The main proc-macro entry point for multi-binding creation.
#[proc_macro]
pub fn create_view_bindings(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as CreateViewBindingsInput);
    parsed.expand().into()
}

/// Top-level input for `create_view_bindings!`.
/// Example usage:
/// ```ignore
/// create_view_bindings!(
///     some_view_expr,
///     {
///         SomeBindings => {
///             {
///                 captures = [capture1.clone(), capture2.clone()],
///                 on_callback(arg: i32) => Self::target_fn
///             },
///             ...
///         },
///         ...
///     }
/// );
/// ```
struct CreateViewBindingsInput {
    view_expr: Expr,
    groups: Vec<BindingGroup>,
}

impl Parse for CreateViewBindingsInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let view_expr: Expr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let content;
        braced!(content in input);

        let mut groups = Vec::new();
        while !content.is_empty() {
            let group: BindingGroup = content.parse()?;
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
            groups.push(group);
        }

        Ok(Self { view_expr, groups })
    }
}

impl CreateViewBindingsInput {
    /// Generates the final macro expansion.
    fn expand(&self) -> proc_macro2::TokenStream {
        let view_expr = &self.view_expr;
        let groups_code = self.groups.iter().map(|group| group.expand());

        quote! {
            #view_expr.execute_on_ui_thread(move |main_window_view, view_binding| {
                #(#groups_code)*
            });
        }
    }
}

/// Represents each binding group, e.g. `SomeBindings => { ... }`.
struct BindingGroup {
    group_name: Path,
    callbacks: Vec<CallbackDefinition>,
}

impl Parse for BindingGroup {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let group_name: Path = input.parse()?;
        input.parse::<Token![=>]>()?;

        let callbacks_block;
        braced!(callbacks_block in input);

        let mut callbacks = Vec::new();
        while !callbacks_block.is_empty() {
            let callback_def: CallbackDefinition = callbacks_block.parse()?;
            if callbacks_block.peek(Token![,]) {
                callbacks_block.parse::<Token![,]>()?;
            }
            callbacks.push(callback_def);
        }

        Ok(Self { group_name, callbacks })
    }
}

impl BindingGroup {
    /// Expands one group of callbacks.
    fn expand(&self) -> proc_macro2::TokenStream {
        let group_name = &self.group_name;
        let callback_defs = self.callbacks.iter().map(|cb| cb.expand());

        quote! {
            {
                let group_bindings = main_window_view.global::<#group_name>();
                #(#callback_defs)*
            }
        }
    }
}

/// Represents one callback definition within a group.
struct CallbackDefinition {
    captures: Vec<Expr>,
    callback_name: Ident,
    args: Vec<(Ident, Ident)>,
    return_type: Option<Ident>,
    target_fn: Path,
}

impl Parse for CallbackDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        braced!(content in input);

        let mut captures = Vec::new();
        let mut callback_name: Option<Ident> = None;
        let mut args: Vec<(Ident, Ident)> = Vec::new();
        let mut return_type: Option<Ident> = None;
        let mut target_fn: Option<Path> = None;

        while !content.is_empty() {
            if content.peek(Ident) {
                let ident_peek: Ident = content.fork().parse()?;
                if ident_peek == "captures" {
                    content.parse::<Ident>()?; // "captures"
                    content.parse::<Token![=]>()?;
                    let bracketed;
                    syn::bracketed!(bracketed in content);
                    let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&bracketed)?;
                    captures = exprs.into_iter().collect();

                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                } else {
                    let (cb_name, cb_args, cb_ret, cb_target) = parse_callback_signature(&&content)?;
                    callback_name = Some(cb_name);
                    args = cb_args;
                    return_type = cb_ret;
                    target_fn = Some(cb_target);
                }
            } else {
                break;
            }
        }

        if callback_name.is_none() || target_fn.is_none() {
            return Err(syn::Error::new_spanned(
                content.parse::<proc_macro2::TokenStream>()?,
                "Missing callback signature.",
            ));
        }

        Ok(Self {
            captures,
            callback_name: callback_name.unwrap(),
            args,
            return_type,
            target_fn: target_fn.unwrap(),
        })
    }
}

/// Parses something like:
///   on_something(arg: Type, arg2: Type) -> ReturnType => Self::some_fn
fn parse_callback_signature(input: &ParseStream) -> SynResult<(Ident, Vec<(Ident, Ident)>, Option<Ident>, Path)> {
    let callback_name: Ident = input.parse()?;

    let args_braces;
    parenthesized!(args_braces in input);

    let mut args = Vec::new();
    while !args_braces.is_empty() {
        if args_braces.peek(Ident) {
            let arg_name: Ident = args_braces.parse()?;
            args_braces.parse::<Token![:]>()?;
            let arg_type: Ident = args_braces.parse()?;
            args.push((arg_name, arg_type));

            if args_braces.peek(Token![,]) {
                args_braces.parse::<Token![,]>()?;
            }
        } else {
            break;
        }
    }

    let mut return_type = None;
    if input.peek(Token![->]) {
        input.parse::<Token![->]>()?;
        let rt: Ident = input.parse()?;
        return_type = Some(rt);
    }

    input.parse::<Token![=>]>()?;
    let target_fn: Path = input.parse()?;

    Ok((callback_name, args, return_type, target_fn))
}

impl CallbackDefinition {
    /// Generates the code for a single callback definition.
    fn expand(&self) -> proc_macro2::TokenStream {
        let callback_name = &self.callback_name;
        let target_fn = &self.target_fn;

        // Create locals for captures
        let captures_lets = self.captures.iter().enumerate().map(|(i, cap_expr)| {
            let cap_var = format_ident!("__cap_{}", i);
            quote! { let #cap_var = #cap_expr; }
        });

        let cap_vars = (0..self.captures.len())
            .map(|i| format_ident!("__cap_{}", i))
            .collect::<Vec<_>>();

        // Generate closure arguments, e.g. |arg: Type, ...|
        let arg_patterns = self.args.iter().map(|(arg_name, arg_type)| {
            quote! { #arg_name: #arg_type }
        });

        // Attach optional return type
        let maybe_return = if let Some(rt) = &self.return_type { quote!(-> #rt) } else { quote!() };

        // Build function call inside the closure
        let call_args = if self.captures.is_empty() {
            if self.args.is_empty() {
                quote!(#target_fn())
            } else {
                let arg_names = self.args.iter().map(|(arg_name, _)| quote!(#arg_name));
                quote!(#target_fn(#(#arg_names),*))
            }
        } else {
            let cloned_caps = cap_vars.iter().map(|var| quote!(#var.clone()));
            if self.args.is_empty() {
                quote!(#target_fn(#(#cloned_caps),*))
            } else {
                let arg_names = self.args.iter().map(|(arg_name, _)| quote!(#arg_name));
                quote!(#target_fn(#(#cloned_caps),*, #(#arg_names),*))
            }
        };

        quote! {
            {
                #(#captures_lets)*

                group_bindings.#callback_name(move |#(#arg_patterns),*| #maybe_return {
                    #call_args
                });
            }
        }
    }
}
