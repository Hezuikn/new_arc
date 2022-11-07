use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, PathArguments};

#[proc_macro_derive(Table, attributes(name_table))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let mut output = quote! {};
    let fields = match &input.data {
        syn::Data::Struct(x) => match &x.fields {
            syn::Fields::Named(x) => &x.named,
            _ => panic!(),
        },
        _ => panic!(),
    };
    for field in fields.iter().rev() {
        let name = &field.ident;
        let ty = match &field.ty {
            syn::Type::Path(x) => {
                let x = x
                    .path
                    .segments
                    .iter()
                    .at_most_one()
                    .unwrap_or_else(|_| panic!())
                    .unwrap();
                assert!(matches!(x.arguments, PathArguments::None));
                let ident = &x.ident;
                let x = ident.to_string();
                let x = match x.as_str() {
                    "i64" => "BigInt",
                    "String" => "Text",
                    "f32" => "Float",
                    "bool" => "Bool",
                    _ => panic!("unknown type: {x}"),
                };
                syn::Ident::new(x, ident.span())
            }
            _ => panic!(),
        };
        //field.attrs
        output = quote! {
            #name -> #ty,
            #output
        };
    }
    let ident = &input
        .attrs
        .iter()
        .find_map(|x| {
            //dbg!(quote!{#x});
            //dbg!(x.tokens.to_string());
            //x.parse_args()
            if let Ok(Some(seg)) = x.path.segments.iter().at_most_one() {
                if seg.ident.to_string() == "name_table" {
                    let ident = x.parse_args().unwrap();
                    //dbg!(&ident);
                    return Some(syn::parse2::<syn::Ident>(ident).unwrap());
                }
            }
            None
        })
        .unwrap();
    let first_as_key = &fields.first().unwrap().ident.as_ref().unwrap();
    let ret = quote! {
        diesel::table! {
            #ident (#first_as_key) {
                #output
            }
        }
    };
    //eprintln!("{}", ret.to_string());
    return ret.into();
}
