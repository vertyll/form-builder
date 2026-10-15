//! `#[derive(Form)]`: a struct asked for field by field in the terminal, through
//! [`form_builder`](https://docs.rs/form_builder). Use it through the `derive` feature of `form_builder`.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Fields, GenericArgument, Ident, LitStr, PathArguments, Type, parse_macro_input};

/// Implements `form_builder::FromForm` for a struct with named fields, one form field for each.
///
/// The type of a field decides what is asked: `bool` a yes-or-no question, `Option<T>` a text field that may be left
/// empty, and any other type a required text field parsed into it. Each field takes `#[form(...)]` with:
///
/// - `prompt = "..."`: the prompt, by default the field's name as words, as `First name:` for `first_name`;
/// - `hint = "..."`: a hint shown with the prompt;
/// - `default = expr`: the answer an empty input gives;
/// - `password`: asks a `String` without showing it, and `repeat = "..."` asks it a second time;
/// - `skip`: leaves the field out of the form, filled with `Default::default()`.
#[proc_macro_derive(Form, attributes(form))]
pub fn derive_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// What `#[form(...)]` says of one field.
#[derive(Default)]
struct Options {
    prompt: Option<LitStr>,
    hint: Option<LitStr>,
    default: Option<Expr>,
    repeat: Option<LitStr>,
    password: bool,
    skip: bool,
}

impl Options {
    fn parse(field: &syn::Field) -> syn::Result<Self> {
        let mut options = Self::default();
        for attribute in field.attrs.iter().filter(|attribute| attribute.path().is_ident("form")) {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("prompt") {
                    options.prompt = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("hint") {
                    options.hint = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("default") {
                    options.default = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("repeat") {
                    options.repeat = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("password") {
                    options.password = true;
                } else if meta.path.is_ident("skip") {
                    options.skip = true;
                } else {
                    return Err(meta.error("expected `prompt`, `hint`, `default`, `password`, `repeat` or `skip`"));
                }
                Ok(())
            })?;
        }
        if options.repeat.is_some() && !options.password {
            return Err(syn::Error::new(field.span(), "`repeat` goes with `password`"));
        }
        Ok(options)
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "`Form` is derived for a struct with named fields",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new(
            input.span(),
            "`Form` is derived for a struct with named fields",
        ));
    };
    let mut fields = Vec::new();
    let mut values = Vec::new();
    for field in &named.named {
        let ident = field.ident.as_ref().expect("a named field has a name");
        let options = Options::parse(field)?;
        if options.skip {
            values.push(quote! { #ident: ::core::default::Default::default() });
            continue;
        }
        let name = ident.to_string();
        let ty = &field.ty;
        fields.push(field_builder(ident, ty, &options)?);
        values.push(quote! { #ident: form.get::<#ty>(#name)? });
    }
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::form_builder::FromForm for #ident #ty_generics #where_clause {
            fn form() -> ::form_builder::FormBuilder {
                ::form_builder::FormBuilder::new() #(.field(#fields))*
            }

            fn from_form(form: &::form_builder::Form) -> ::core::result::Result<Self, ::form_builder::Error> {
                ::core::result::Result::Ok(Self { #(#values),* })
            }
        }
    })
}

/// The builder of the form field that asks for `ident`, of type `ty`.
fn field_builder(ident: &Ident, ty: &Type, options: &Options) -> syn::Result<TokenStream2> {
    let name = ident.to_string();
    let prompt = options
        .prompt
        .clone()
        .unwrap_or_else(|| LitStr::new(&format!("{}:", words(&name)), Span::call_site()));
    let mut builder = if options.password {
        let repeat = options.repeat.as_ref().map(|repeat| quote! { .repeat(#repeat) });
        quote! { ::form_builder::Password::new(#name, #prompt) #repeat }
    } else if is_bool(ty) {
        quote! { ::form_builder::Confirm::new(#name, #prompt) }
    } else if let Some(inner) = option_inner(ty) {
        quote! { ::form_builder::Text::<#inner>::optional(#name, #prompt) }
    } else {
        quote! { ::form_builder::Text::<#ty>::new(#name, #prompt) }
    };
    if let Some(default) = &options.default {
        if options.password {
            return Err(syn::Error::new(default.span(), "a password has no default"));
        }
        builder = quote! { #builder.default(#default) };
    }
    if let Some(hint) = &options.hint {
        builder = quote! { #builder.hint(#hint) };
    }
    Ok(builder)
}

/// `name` as words, its first letter capital: `first_name` as `First name`.
fn words(name: &str) -> String {
    let spaced = name.trim_start_matches("r#").replace('_', " ");
    let mut chars = spaced.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn last_segment(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last(),
        _ => None,
    }
}

fn is_bool(ty: &Type) -> bool {
    last_segment(ty).is_some_and(|segment| segment.ident == "bool" && segment.arguments.is_none())
}

/// The `T` of `Option<T>`, when `ty` is one.
fn option_inner(ty: &Type) -> Option<&Type> {
    let segment = last_segment(ty).filter(|segment| segment.ident == "Option")?;
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{is_bool, option_inner, words};

    #[test]
    fn a_name_reads_as_words() {
        assert_eq!(words("first_name"), "First name");
        assert_eq!(words("r#type"), "Type");
        assert_eq!(words("age"), "Age");
    }

    #[test]
    fn the_type_decides_the_kind_of_field() {
        assert!(is_bool(&parse_quote!(bool)));
        assert!(!is_bool(&parse_quote!(u8)));
        assert!(option_inner(&parse_quote!(Option<u16>)).is_some());
        assert!(option_inner(&parse_quote!(std::option::Option<String>)).is_some());
        assert!(option_inner(&parse_quote!(Vec<u16>)).is_none());
    }
}
