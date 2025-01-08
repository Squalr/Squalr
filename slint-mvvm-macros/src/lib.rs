use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprClosure, Ident, Path, Result as SynResult, Token,
};

// -------------------------------------------------------------------------------------
// 1) Top-level macro entry
// -------------------------------------------------------------------------------------
#[proc_macro]
pub fn create_view_bindings(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as CreateViewBindingsInput);
    parsed.expand().into()
}

struct CreateViewBindingsInput {
    view_expr: Expr,
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
    fn expand(&self) -> proc_macro2::TokenStream {
        let view_expr = &self.view_expr;
        let groups_code = self.groups.iter().map(|group| group.expand());

        quote! {
            {
                let __view_binding = #view_expr.clone();
                __view_binding.execute_on_ui_thread(move |main_window_view, view_binding| {
                    #(#groups_code)*
                });
            }
        }
    }
}

// -------------------------------------------------------------------------------------
// 2) BindingGroup
// -------------------------------------------------------------------------------------
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

// -------------------------------------------------------------------------------------
// 3) CallbackDefinition (supports a Path or a Closure)
// -------------------------------------------------------------------------------------
struct CallbackDefinition {
    callback_name: Ident,
    // We still parse function-like arguments from `(foo: Type, bar: Type)`,
    // but we might ignore them if the user writes a closure after `->`.
    args: Vec<(Ident, Ident)>,
    target: CallbackTarget,
    captures: Vec<Expr>,
}

enum CallbackTarget {
    Path(Path),
    Closure(ExprClosure),
}

impl Parse for CallbackDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Example lines:
        //   on_drag(delta_x: i32, delta_y: i32) -> Self::on_drag [view_binding.clone()]
        //   on_required_write_changed(value: bool) -> |value| { ... } [captures]

        // 1) Parse callback_name (e.g. `on_drag`)
        let callback_name: Ident = input.parse()?;

        // 2) Parse parentheses with arguments if any: (delta_x: i32, delta_y: i32)
        let args_braces;
        parenthesized!(args_braces in input);
        let args = parse_args(&args_braces)?;

        // 3) Parse '->'
        input.parse::<Token![->]>()?;

        // 4) Decide if it's a Path or a Closure
        let lookahead = input.lookahead1();
        let target = if lookahead.peek(Token![|]) {
            // It's a lambda-style closure
            let expr_closure: ExprClosure = input.parse()?;
            CallbackTarget::Closure(expr_closure)
        } else {
            // It's a path style: e.g. Self::something
            let path: Path = input.parse()?;
            CallbackTarget::Path(path)
        };

        // 5) Optionally parse bracketed captures: `[expr, expr, ...]`
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
            target,
            captures,
        })
    }
}

// Helper to parse a comma-separated list of `(ident: Type)` pairs
fn parse_args(input: ParseStream) -> SynResult<Vec<(Ident, Ident)>> {
    let mut args = Vec::new();

    while !input.is_empty() {
        let arg_name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let arg_type: Ident = input.parse()?;
        args.push((arg_name, arg_type));

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(args)
}

// -------------------------------------------------------------------------------------
// 4) Expansion
// -------------------------------------------------------------------------------------
impl CallbackDefinition {
    fn expand(&self) -> proc_macro2::TokenStream {
        let callback_name = &self.callback_name;
        let captures_lets = self.captures.iter().enumerate().map(|(i, cap_expr)| {
            let cap_var = format_ident!("__cap_{}", i);
            quote! {
                let #cap_var = #cap_expr.clone();
            }
        });

        let cap_vars = (0..self.captures.len())
            .map(|i| format_ident!("__cap_{}", i))
            .collect::<Vec<_>>();

        // We still have (arg_name, arg_type) from the parentheses, though in closure mode,
        // we might simply ignore them or unify them. For path style, we need them.
        let arg_patterns = self
            .args
            .iter()
            .map(|(arg_name, arg_type)| quote! { #arg_name: #arg_type });

        match &self.target {
            // ---------------------------------------------------------------------------------
            // Path-based target: e.g. `-> Self::some_fn`
            // ---------------------------------------------------------------------------------
            CallbackTarget::Path(path) => {
                // e.g. Self::on_drag
                let fn_call = if self.captures.is_empty() {
                    // No captures
                    if self.args.is_empty() {
                        // No arguments
                        quote!(#path())
                    } else {
                        // Just arguments
                        let arg_names = self.args.iter().map(|(name, _)| quote!(#name));
                        quote!(#path(#(#arg_names),*))
                    }
                } else {
                    // We have captures
                    let cloned_caps = cap_vars.iter().map(|v| quote!(#v.clone()));
                    if self.args.is_empty() {
                        // Only captures
                        quote!(#path(#(#cloned_caps),*))
                    } else {
                        // Both captures and arguments
                        let arg_names = self.args.iter().map(|(name, _)| quote!(#name));
                        quote!(#path(#(#cloned_caps),*, #(#arg_names),*))
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

            // ---------------------------------------------------------------------------------
            // Closure-based target: e.g. `-> |value| { ... }`
            // ---------------------------------------------------------------------------------
            CallbackTarget::Closure(expr_closure) => {
                // We don’t need to generate a closure in a closure.
                // We can directly pass the user’s closure to `group_bindings.<callback_name>()`,
                // but we still wrap it in `move` and handle captures.
                //
                // NOTE: If you want to enforce that the user must define the same arguments
                // in the closure as in `(foo: Type)`, you can add checks or unify them here.
                // For now, we just ignore the parentheses-based arguments.
                quote! {
                    {
                        #(#captures_lets)*

                        group_bindings.#callback_name(
                            // We capture everything, then call the user’s closure
                            move #expr_closure
                        );
                    }
                }
            }
        }
    }
}
