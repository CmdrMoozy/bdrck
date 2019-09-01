// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Expr, FnArg, GenericArgument, Ident, ItemFn, Pat, PathArguments, Type,
    TypeSlice,
};

// A special case of `handle_argument`, for command callback arguments which are
// guaranteed to have exactly one value (this includes required and boolean
// flags).
fn handle_required_argument(name: &Ident, ty: &Type) -> (TokenStream, Expr) {
    (
        TokenStream::from(quote! {
            let #name: #ty = take_required(values.get_as(stringify!(#name))?)?;
        }),
        Expr::Verbatim(TokenStream::from(quote! { #name })),
    )
}

// A special case of `handle_argument`, for command callback arguments which are
// optional (i.e., which will have exactly 0 or 1 values). This refers mainly to
// optional flags which can be omitted entirely.
fn handle_optional_argument(name: &Ident, inner_ty: &Type) -> (TokenStream, Expr) {
    (
        TokenStream::from(quote! {
            let #name: Option<#inner_ty> = take_optional(values.get_as(stringify!(#name))?)?;
        }),
        Expr::Verbatim(TokenStream::from(quote! { #name })),
    )
}

// A special case of `handle_argument`, for command callback arguments of the
// form `&[T]`. These should be positional arguments which can have zero or more
// values.
fn handle_slice_argument(name: &Ident, slice: &TypeSlice) -> (TokenStream, Expr) {
    let elem = &slice.elem;
    (
        TokenStream::from(quote! {
            let #name: Vec<#elem> = values.get_as(stringify!(#name))?;
        }),
        Expr::Verbatim(TokenStream::from(quote! { #name.as_slice() })),
    )
}

// Handles a single callback function argument by generating the code for two
// pieces (returned separately in a tuple):
//
// - The code which extracts the argument from the monolithic `Values` struct.
// - The `Expr` which we'll pass in to the real command callback with the value.
fn handle_argument(arg: &FnArg) -> (TokenStream, Expr) {
    match arg {
        FnArg::Receiver(_) => panic!("Command callbacks cannot be member functions."),
        FnArg::Typed(pattern_type) => {
            match &*pattern_type.pat {
                Pat::Ident(ident) => {
                    let ty = &pattern_type.ty;
                    let name = &ident.ident;

                    match &**ty {
                        Type::Reference(r) => match r.elem.as_ref() {
                            Type::Slice(s) => handle_slice_argument(name, s),
                            _ => panic!("Command callbacks only accept references of slices."),
                        },
                        Type::Path(p) => {
                            if let Some(last) = p.path.segments.last() {
                                // This is gross, but since we're dealing with an
                                // AST there isn't another way to differentiate
                                // between Option and other types...
                                if last.ident.to_string() == "Option" {
                                    match &last.arguments {
                                        PathArguments::AngleBracketed(inner_ty) => {
                                            if inner_ty.args.len() != 1 {
                                                panic!("Found unrecognized Option type with multiple generic type arguments");
                                            }
                                            match inner_ty.args.last().unwrap() {
                                            GenericArgument::Type(inner_ty) => return handle_optional_argument(name, &inner_ty),
                                            _ => panic!("Found unrecognized Option generic type parameter"),
                                        }
                                        }
                                        _ => panic!("Found unrecognized, non-generic Option type"),
                                    }
                                }
                            }

                            handle_required_argument(name, &ty)
                        }
                        _ => panic!("Invalid command callback parameter type"),
                    }
                }
                _ => panic!("Command callback function takes an unhandled argument type."),
            }
        }
    }
}

#[proc_macro_attribute]
pub fn command_callback(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // This attribute macro accepts no arguments.
    assert!(args.is_empty());

    let input = parse_macro_input!(input as ItemFn);
    let visibility = input.vis;
    let constness = input.sig.constness;
    let unsafety = input.sig.unsafety;
    let asyncness = input.sig.asyncness;
    let abi = input.sig.abi;
    let name = input.sig.ident;

    let args = input.sig.inputs;
    let output = input.sig.output;

    let block = input.block;

    let mut arg_parsing = TokenStream::new();
    let mut real_args: Punctuated<Expr, Comma> = Punctuated::new();

    for arg in args.iter() {
        let (parse, arg) = handle_argument(arg);
        arg_parsing.extend(parse);
        real_args.push(arg);
    }

    TokenStream::from(quote! {
        #visibility #constness #unsafety #asyncness #abi fn #name(values: Values) #output {
            #arg_parsing
            #constness #unsafety #asyncness #abi fn inner(#args) #output #block
            inner(#real_args)
        }
    })
    .into()
}
