extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(ArtifishExpr, attributes(expr_tree_node))]
pub fn derive_artifish_expr(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let data = match input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("expected a struct"),
    };

    let struct_ident = input.ident;
    let struct_generics = input.generics;

    let mut child_exprs = Vec::new();

    for field in data.fields.iter() {
        // You can set an attribute on an Expr struct field that makes us
        // consider this field not a child expression, e.g.
        // ```
        // struct ConstExpr<T> {
        //     #[expr_tree_node(not_a_child)]
        //     value: T
        // }
        // ```
        let mut skip_field = false;
        for attribute in field.attrs.iter() {
            if !attribute.path.is_ident("expr_tree_node") {
                continue;
            }

            match attribute.parse_args::<Ident>() {
                Ok(ident) if ident == "not_a_child" => {
                    skip_field = true;
                }
                Ok(ident) => panic!("Shit input {}", ident.to_string()),
                Err(_) => panic!("Shit input {:?}", attribute.tokens),
            }
        }

        if !skip_field {
            child_exprs.push(field);
        }
    }

    let num_children = child_exprs.len() as u64;

    let child_match_entries = child_exprs
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let i = i as u64;
            // TODO: no unwrap
            let ident = field.ident.as_ref().unwrap();
            quote! {
                #i => &self.#ident,
            }
        })
        .collect::<Vec<_>>();

    let child_match_entries_mut = child_exprs
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let i = i as u64;
            // TODO: no unwrap
            let ident = field.ident.as_ref().unwrap();
            quote! {
                #i => &mut self.#ident,
            }
        })
        .collect::<Vec<_>>();

    let tokens = quote! {
        impl #struct_generics ExprTreeNode for #struct_ident #struct_generics {
            fn num_children(&self) -> u64 {
                #num_children
            }

            fn borrow_nth_child(&self, n: u64) -> &dyn MutableExprSlot {
                match n {
                    #( #child_match_entries )*
                    _ => panic!("child index out of range"),
                }
            }

            fn borrow_nth_child_mut(&mut self, n: u64) -> &mut dyn MutableExprSlot {
                match n {
                    #( #child_match_entries_mut )*
                    _ => panic!("child index out of range"),
                }
            }
        }
    };

    return tokens.into();
}
