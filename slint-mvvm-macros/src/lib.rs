// Usage example:
// create_view_bindings!(
//     view_binding,
//     {
//         SomeBindings => {
//             on_minimize() -> Self::on_minimize [capture1.clone()],
//             on_maximize() -> Self::on_maximize [capture1.clone()],
//             on_close() -> Self::on_close [],
//             on_drag(delta_x: i32, delta_y: i32) -> Self::on_drag [capture2.clone()]
//         },
//         AnotherBindings => {
//             on_callback(arg: i32) -> Self::other_callback [capture1.clone(), capture2.clone()]
//         }
//     }
// );
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

/// The main proc-macro entry point for multi-binding creation,
/// supporting the **NEW** format only:
///
/// ```ignore
/// create_view_bindings!(
///     some_view_expr,
///     {
///         SomeBindings => {
///             on_minimize() -> Self::on_minimize [view_binding.clone()],
///             on_maximize() -> Self::on_maximize [view_binding.clone()],
///             on_close() -> Self::on_close [],
///             on_drag(delta_x: i32, delta_y: i32) -> Self::on_drag [view_binding.clone()]
///         },
///         ...
///     }
/// );
/// ```
#[proc_macro]
pub fn create_view_bindings(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as CreateViewBindingsInput);
    parsed.expand().into()
}

/// Top-level input for `create_view_bindings!`.
struct CreateViewBindingsInput {
    /// The expression (e.g. `view_binding.clone()`) that will call `execute_on_ui_thread`.
    view_expr: Expr,
    /// One or more binding groups `{ MyBindings => { ... }, OtherBindings => { ... } }`.
    groups: Vec<BindingGroup>,
}

impl Parse for CreateViewBindingsInput {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // First parse the expression (e.g. `view_binding.clone()`).
        let view_expr: Expr = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // Then parse the braces containing the binding groups.
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
    /// Generates the final macro expansion that calls `execute_on_ui_thread`
    /// once, and places each binding group inside.
    fn expand(&self) -> proc_macro2::TokenStream {
        let view_expr = &self.view_expr;
        let groups_code = self.groups.iter().map(|group| group.expand());

        // Instead of:
        //
        // quote! {
        //     #view_expr.execute_on_ui_thread(move |main_window_view, view_binding| {
        //         #(#groups_code)*
        //     });
        // }
        //
        // do something like this:
        quote! {
            {
                // Force a clone here
                let __view_binding = #view_expr.clone();
                __view_binding.execute_on_ui_thread(move |main_window_view, view_binding| {
                    #(#groups_code)*
                });
            }
        }
    }
}

/// Represents each binding group, e.g. `SomeBindings => { on_minimize() -> Self::on_minimize [caps], ... }`.
struct BindingGroup {
    group_name: Path,
    callbacks: Vec<CallbackDefinition>,
}

impl Parse for BindingGroup {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let group_name: Path = input.parse()?; // e.g. `WindowViewModelBindings`
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
    /// Expands the group into code that fetches the group bindings
    /// and registers each callback definition.
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

/// Represents one callback definition in the new format:
///
/// ```ignore
/// on_something(arg: Type, arg2: Type) -> Self::my_target_fn [cap1.clone(), cap2.clone()]
/// ```
struct CallbackDefinition {
    callback_name: Ident,
    args: Vec<(Ident, Ident)>,
    target_fn: Path,
    captures: Vec<Expr>,
}

impl Parse for CallbackDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Example line:
        // on_drag(delta_x: i32, delta_y: i32) -> Self::on_drag [view_binding.clone()]

        // 1) Parse callback_name (e.g. `on_drag`)
        let callback_name: Ident = input.parse()?;

        // 2) Parse parentheses with arguments (e.g. `(delta_x: i32, delta_y: i32)`)
        let args_braces;
        parenthesized!(args_braces in input);
        let args = parse_args(&args_braces)?;

        // 3) Parse `->`
        input.parse::<Token![->]>()?;

        // 4) Parse the path to the target function (e.g. `Self::on_drag`)
        let target_fn: Path = input.parse()?;

        // 5) Parse optional bracketed captures: `[expr, expr, ...]`
        //    Use `syn::token::Bracket` instead of `Token!['[']`.
        let captures = if input.peek(syn::token::Bracket) {
            let bracketed;
            syn::bracketed!(bracketed in input);
            let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&bracketed)?;
            exprs.into_iter().collect()
        } else {
            vec![]
        };

        Ok(Self {
            callback_name,
            args,
            target_fn,
            captures,
        })
    }
}

/// Helper to parse a comma-separated list of `(ident: Type)` pairs.
fn parse_args(input: ParseStream) -> SynResult<Vec<(Ident, Ident)>> {
    let mut args = Vec::new();

    while !input.is_empty() {
        // e.g. `delta_x: i32`
        let arg_name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let arg_type: Ident = input.parse()?;
        args.push((arg_name, arg_type));

        // If there's another comma, consume it.
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(args)
}

impl CallbackDefinition {
    /// Expands into the code that will register the callback with the given captures and arguments.
    fn expand(&self) -> proc_macro2::TokenStream {
        let callback_name = &self.callback_name;
        let target_fn = &self.target_fn;

        // Create locals for captures, e.g.:
        // let __cap_0 = view_binding.clone();
        let captures_lets = self.captures.iter().enumerate().map(|(i, cap_expr)| {
            let cap_var = format_ident!("__cap_{}", i);
            quote! {
                let #cap_var = #cap_expr.clone();
            }
        });

        // Prepare the captured variables for calling the function
        let cap_vars = (0..self.captures.len())
            .map(|i| format_ident!("__cap_{}", i))
            .collect::<Vec<_>>();

        // Generate closure arguments: |arg: Type, ...|
        let arg_patterns = self.args.iter().map(|(arg_name, arg_type)| {
            quote! { #arg_name: #arg_type }
        });

        // Generate the function call inside the closure
        let fn_call = if self.captures.is_empty() {
            // No captures
            if self.args.is_empty() {
                // No arguments
                quote!(#target_fn())
            } else {
                // Just arguments
                let arg_names = self.args.iter().map(|(arg_name, _)| quote!(#arg_name));
                quote!(#target_fn(#(#arg_names),*))
            }
        } else {
            // We have captures
            let cloned_caps = cap_vars.iter().map(|var| quote!(#var.clone()));
            if self.args.is_empty() {
                // Captures, no arguments
                quote!(#target_fn(#(#cloned_caps),*))
            } else {
                // Both captures and arguments
                let arg_names = self.args.iter().map(|(arg_name, _)| quote!(#arg_name));
                quote!(#target_fn(#(#cloned_caps),*, #(#arg_names),*))
            }
        };

        quote! {
            {
                #(#captures_lets)*

                group_bindings.#callback_name(move |#(#arg_patterns),*| {
                    #fn_call
                });
            }
        }
    }
}
