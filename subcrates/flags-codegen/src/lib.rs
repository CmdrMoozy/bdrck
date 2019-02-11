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
use syn::{parse_macro_input, Expr, ExprVerbatim, FnArg, Ident, ItemFn, Pat, Type, TypeSlice};

// A special case of `handle_argument`, for command callback arguments which are
// guaranteed to have exactly one value (this includes required and boolean
// flags).
fn handle_required_argument(name: &Ident, ty: &Type) -> (TokenStream, Expr) {
    (
        TokenStream::from(quote! {
            let #name: #ty = take_required(values.get_as(stringify!(#name))?)?;
        }),
        Expr::Verbatim(ExprVerbatim {
            tts: TokenStream::from(quote! { #name }),
        }),
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
        Expr::Verbatim(ExprVerbatim {
            tts: TokenStream::from(quote! { #name.as_slice() }),
        }),
    )
}

// Handles a single callback function argument by generating the code for two
// pieces (returned separately in a tuple):
//
// - The code which extracts the argument from the monolithic `Values` struct.
// - The `Expr` which we'll pass in to the real command callback with the value.
fn handle_argument(arg: &FnArg) -> (TokenStream, Expr) {
    match arg {
        FnArg::Captured(captured) => match &captured.pat {
            Pat::Ident(ident) => {
                let ty = &captured.ty;
                let name = &ident.ident;

                match ty {
                    Type::Reference(r) => match r.elem.as_ref() {
                        Type::Slice(s) => handle_slice_argument(name, s),
                        _ => panic!("Command callbacks only accept references of slices."),
                    },
                    _ => handle_required_argument(name, ty),
                }
            }
            _ => panic!("Command callback function takes an unhandled argument type."),
        },
        FnArg::SelfRef(_) => panic!("Command callbacks cannot be member functions."),
        FnArg::SelfValue(_) => panic!("Command callbacks cannot be member functions."),
        FnArg::Inferred(_) => panic!("Command callbacks cannot be lambdas with inferred captures."),
        FnArg::Ignored(_) => panic!("Command callbacks cannot have ignored arguments."),
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
    let constness = input.constness;
    let unsafety = input.unsafety;
    let asyncness = input.asyncness;
    let abi = input.abi;
    let name = input.ident;

    let args = input.decl.inputs;
    let output = input.decl.output;

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
