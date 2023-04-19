use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, PathArguments};

#[proc_macro_derive(Table, attributes(name_table))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let mut output = quote! {};
    let fields = match input.data {
        syn::Data::Struct(x) => match x.fields {
            syn::Fields::Named(x) => x.named,
            _ => panic!("0"),
        },
        _ => panic!("1"),
    };
    let first_as_key = fields.first().unwrap().ident.as_ref().unwrap().to_owned();
    for field in fields.into_iter().rev() {
        let name = &field.ident;
        let ty_span = field.ty.span();
        let ty = match &field.ty {
            syn::Type::Path(type_path) => {
                let path_segment = type_path
                    .path
                    .segments
                    .iter()
                    .at_most_one()
                    .unwrap_or_else(|_| panic!("2"))
                    .unwrap();
                assert!(matches!(path_segment.arguments, PathArguments::None));
                let ident = &path_segment.ident;
                let ident_str = ident.to_string();
                let sql_ident_str = match ident_str.as_str() {
                    "i64" => "BigInt",
                    "String" => "Text",
                    "f32" => "Float",
                    "bool" => "Bool",
                    "NaiveDateTime" => "Timestamp",
                    _ => panic!("/unknown type: {ident_str}/"),
                };
                syn::Type::Path(syn::parse_str(sql_ident_str).unwrap())
            }
            _ => field.ty,
        };
        //field.attrs
        output = quote_spanned! {
            ty_span =>
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
            if let Ok(Some(seg)) = x.path().segments.iter().at_most_one() {
                if seg.ident.to_string() == "name_table" {
                    let ident = x.parse_args().unwrap();
                    //dbg!(&ident);
                    return Some(syn::parse2::<syn::Ident>(ident).unwrap());
                }
            }
            None
        })
        .unwrap();
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
