use darling::{
    ast::{Data, Style},
    FromDeriveInput, FromField,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct StructData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), StructFields>,
}

#[derive(Debug, FromField)]
struct StructFields {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub(crate) fn process_from_js(input: DeriveInput) -> TokenStream {
    let (ident, generics, merged, fields) = parse_struct(input);

    let code = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");
        let ty = &field.ty;

        quote! {
          let #name: #ty = obj.get(stringify!(#name))?;
        }
    });

    let idents = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");
        quote! { #name }
    });

    quote! {
        impl #merged rquickjs::FromJs<'js> for #ident #generics {
            fn from_js(_ctx: &rquickjs::Ctx<'js>, v: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                let obj = v.into_object().unwrap();

                #(#code)*

                Ok(#ident {
                    #(#idents),*
                })
            }
        }
    }
}

pub(crate) fn process_into_js(input: DeriveInput) -> TokenStream {
    let (ident, generics, merged, fields) = parse_struct(input);
    let code = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");

        quote! {
          obj.set(stringify!(#name), self.#name)?;
        }
    });

    quote! {
        impl #merged rquickjs::IntoJs<'js> for #ident #generics {
            fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                let obj = rquickjs::Object::new(ctx.clone())?;

                #(#code)*

                Ok(obj.into())
            }
        }
    }
}

fn parse_struct(
    input: DeriveInput,
) -> (syn::Ident, syn::Generics, syn::Generics, Vec<StructFields>) {
    let StructData {
        ident,
        generics,
        data: Data::Struct(fields),
    } = StructData::from_derive_input(&input).expect("Can not parse input")
    else {
        panic!("Only struct is supported");
    };

    let fields = match fields.style {
        Style::Struct => fields.fields,
        _ => panic!("Only named fields are supported"),
    };

    let mut merged = generics.clone();

    merged.params.push(syn::parse_quote!('js));
    (ident, generics, merged, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_from_js_should_work() {
        let input = r#"
          #[derive(FromJs)]
          struct Request {
            method: String,
            url: String,
            headers: HashMap<String, String>,
            body: Option<String>,
          }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();

        assert_eq!(info.ident.to_string(), "Request");

        let code = process_from_js(parsed);
        println!("{}", code);
    }

    #[test]
    fn process_into_js_should_work() {
        let input = r#"
          #[derive(IntoJs)]
          struct Request {
            method: String,
            url: String,
            headers: HashMap<String, String>,
            body: Option<String>,
          }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();

        assert_eq!(info.ident.to_string(), "Request");

        let code = process_into_js(parsed);
        println!("{}", code);
    }
}
