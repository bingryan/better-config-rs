use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Field, Fields, Meta, Path, Token,
};

struct StructEnvArgs {
    trait_path: Option<Path>,
    prefix: Option<String>,
    target: Option<String>,
    generic_args: Vec<syn::GenericArgument>,
}

/**
 * `env` macro for deriving environment variable loading functionality.
 *
 * ## Example
 * ```rust,ignore
 *  #[env(EnvConfig)]
 *  pub struct Config {
 *       #[conf(from = "PORT", default = "5432", setter = "set_port", getter = "get_port")]
 *       pub port: u16,
 *   }
 *  ```
 *
 */
fn parse_field_env_args(field: &Field, meta: &Meta) -> Vec<Meta> {
    if field.attrs.iter().any(|attr| attr.path().is_ident("env")) {
        return Vec::new();
    }
    match meta {
        // #[env(name = "value", other = "value2")]
        Meta::List(l) => l
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_else(|err| {
                let field_name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| String::from("unnamed"));

                panic!(
                    "Failed to parse env attribute on field `{}`: {:?}",
                    field_name, err
                )
            })
            .iter()
            .cloned()
            .collect(),
        // #[env]
        Meta::Path(_) => Vec::new(),
        // #[env = "value"]
        _ => vec![meta.clone()],
    }
}

// #[env(EnvConfig(prefix = "APP_", target = ".env"))]
fn parse_struct_env_args(args: Meta) -> StructEnvArgs {
    let mut prefix = None;
    let mut target = None;
    let mut generic_args = Vec::new();
    let trait_path;

    match args {
        // #[env(EnvConfig(prefix = "...", ...))]
        Meta::List(meta_list) => {
            trait_path = Some(meta_list.path.clone());

            if let Some(last_segment) = meta_list.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                {
                    generic_args.extend(angle_bracketed.args.iter().cloned());
                }
            }

            let _ = meta_list.parse_nested_meta(|nested_meta| {
                if nested_meta.path.is_ident("prefix") {
                    if let Ok(value) = nested_meta.value()?.parse::<syn::LitStr>() {
                        prefix = Some(value.value());
                    }
                } else if nested_meta.path.is_ident("target") {
                    if let Ok(value) = nested_meta.value()?.parse::<syn::LitStr>() {
                        target = Some(value.value());
                    }
                }
                Ok(())
            });
        }
        // #[env(EnvConfig)]
        Meta::Path(path) => {
            trait_path = Some(path.clone());

            if let Some(last_segment) = path.segments.last() {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                {
                    generic_args.extend(angle_bracketed.args.iter().cloned());
                }
            }
        }
        _ => panic!(
            "Invalid env macro arguments. Expected #[env(EnvConfig)] or #[env(EnvConfig(...))]"
        ),
    }

    // generic_args > 1  => panic!
    if generic_args.len() > 1 {
        panic!("env macro only supports one generic argument");
    }

    StructEnvArgs {
        trait_path,
        prefix,
        target,
        generic_args,
    }
}
#[proc_macro_attribute]
pub fn env(args: TokenStream, input: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(args as Meta);
    let env_args = parse_struct_env_args(meta);

    let input_clone = input.clone();
    let input_ref = parse_macro_input!(input_clone as DeriveInput);
    let struct_name = &input_ref.ident;
    let vis = &input_ref.vis;
    let trait_path = &env_args.trait_path;

    // Extract fields from the input
    let fields = match &input_ref.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("env macro only supports structs"),
    };

    let builder_field_assigns = fields
        .iter()
        .map(|field| handle_builder_field_assign(&env_args, field));

    let field_defs = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: #field_type
        }
    });

    let builder_field_defs = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: Option<#field_type>
        }
    });

    let target = match &env_args.target {
        Some(t) => quote! { Some(#t.to_string()) },
        None => quote! { None },
    };

    let params_type = if env_args.generic_args.is_empty() {
        quote! { ::std::collections::HashMap<String, String> }
    } else {
        let generic_arg = &env_args.generic_args[0];
        quote! { #generic_arg }
    };

    let params_field = quote! {
        _params: ::std::collections::HashMap<String, String>
    };

    let params_new_field = quote! {
        _params: ::std::collections::HashMap::new()
    };

    let helper_trait = quote::format_ident!("{}BetterHelper", struct_name);

    let struct_builder = quote::format_ident!("{}Builder", struct_name);

    let loaded_params_var = quote::format_ident!("loaded_params");
    let field_assigns = handle_field_assigns(fields, &env_args, &loaded_params_var);

    let getter_methods = fields.iter().filter_map(|field| {
        let field_env_attr = field.attrs.iter().find(|attr| attr.path().is_ident("conf"));
        if let Some(field_env_attr) = field_env_attr {
            let field_env_args = parse_field_env_args(field, &field_env_attr.meta);
            for attr in field_env_args {
                if attr.path().is_ident("getter") {
                    if let Meta::NameValue(name_value) = attr {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &name_value.value
                        {
                            let getter_ident = quote::format_ident!("{}", lit_str.value());
                            let field_type = &field.ty;
                            return Some(quote! {
                                fn #getter_ident(&self,#params_field) -> #field_type;
                            });
                        }
                    }
                }
            }
        }
        None
    });

    let expanded = quote! {
        #vis struct #struct_name {
            #params_field,
            #(#field_defs),*,
        }

        impl #struct_name {
            // builder
            pub fn builder() -> #struct_builder {
                #struct_builder::new()
            }
        }


        #vis struct #struct_builder {
            #params_field,
            #(#builder_field_defs),*,
        }

        impl #struct_builder {
            // new
            pub fn new() -> Self {
                Self {
                    #params_new_field,
                    #(#builder_field_assigns),*,
                }
            }
            // builder methods
            pub fn build(&mut self) -> Result<#struct_name, better_config::Error> {
                // load first
                let loaded_params = <Self as #trait_path<#params_type>>::load(#target)?;
                let config = #struct_name {
                    _params: loaded_params.clone(),
                    #(#field_assigns),*,
                };
                Ok(config)
            }
        }

        // generate a help trait to add getter and setter methods
        trait #helper_trait {
            #(#getter_methods)*
        }

        // First implement AbstractConfig
        impl better_config::AbstractConfig<#params_type> for #struct_name {
            fn load(target: Option<String>) -> Result<#params_type, better_config::Error> {
                // Default to calling EnvConfig's load
                <Self as #trait_path<#params_type>>::load(#target)
            }
        }

        impl better_config::AbstractConfig<#params_type> for #struct_builder {
            fn load(target: Option<String>) -> Result<#params_type, better_config::Error> {
                // Default to calling EnvConfig's load
                <Self as #trait_path<#params_type>>::load(#target)
            }
        }

        impl #trait_path<#params_type> for #struct_name  {}
        impl #trait_path<#params_type> for #struct_builder  {}

    };

    TokenStream::from(expanded)
}

fn handle_builder_field_assign(
    env_args: &StructEnvArgs,
    field: &Field,
) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;

    // nested
    let is_nested = field.attrs.iter().any(|attr| attr.path().is_ident("env"));
    if is_nested {
        return quote! {
            #field_name: None
        };
    }

    let from = get_var_name(field, "from");
    let default = get_var_name(field, "default");
    // if from and default are both None, return None
    if from.is_none() && default.is_none() {
        return quote! {
            #field_name: None
        };
    }

    let mut var_name =
        from.unwrap_or_else(|| field_name.as_ref().unwrap().to_string().to_uppercase());

    // Add prefix if specified
    if let Some(ref prefix) = env_args.prefix {
        var_name = format!("{}{}", prefix, var_name);
    }

    if let Some(default) = default {
        return quote! {
            #field_name: ::better_config::utils::env::get_optional_or::<_,#field_type>(#var_name, #default.parse::<#field_type>().unwrap())
        };
    }

    quote! {
            #field_name: ::better_config::utils::env::get_optional::<_,#field_type>(#var_name)
    }
}

fn handle_field_assign(
    env_args: &StructEnvArgs,
    field: &Field,
    loaded_params_var: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    // get field conf attr
    let field_env_attr = field.attrs.iter().find(|attr| attr.path().is_ident("conf"));

    let field_name = &field.ident;

    let is_nested = field.attrs.iter().any(|attr| attr.path().is_ident("env"));

    if is_nested {
        let field_type = &field.ty;
        return quote! {
            #field_name: #field_type::builder()
                .build()
                .expect("Failed to build nested config")
        };
    }

    let assign = if let Some(field_env_attr) = field_env_attr {
        match &field_env_attr.meta {
            Meta::List(_) => handle_field_meta_list(env_args, field, loaded_params_var),
            _ => panic!(
                "Unsupported env attribute on field `{}`",
                field_name.as_ref().unwrap()
            ),
        }
    } else {
        let field_name_str = field_name.as_ref().unwrap().to_string().to_uppercase();
        quote! {
            #field_name: ::better_config::utils::env::get_or_else(#field_name_str, || panic!("Failed to load from var: {}", #field_name_str))?
        }
    };

    quote! {
        #assign
    }
}

fn handle_field_meta_list(
    env_args: &StructEnvArgs,
    field: &Field,
    loaded_params_var: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;

    let mut var_name = get_var_name(field, "from")
        .unwrap_or_else(|| field_name.as_ref().unwrap().to_string().to_uppercase());

    // Add prefix if specified
    if let Some(ref prefix) = env_args.prefix {
        var_name = format!("{}{}", prefix, var_name);
    }

    // handle attributes
    let default = get_var_name(field, "default");
    let setter_name = get_var_name(field, "setter");
    let getter_name = get_var_name(field, "getter");

    if let Some(getter) = getter_name {
        let getter_ident = quote::format_ident!("{}", getter);
        quote! {
            #field_name: <Self>::#getter_ident(&self,&#loaded_params_var)
        }
    } else if let Some(setter) = setter_name {
        let setter_ident = quote::format_ident!("{}", setter);
        quote! {
            #field_name: {
                let value = #loaded_params_var.get(#var_name).cloned().unwrap_or_default();
                self.#setter_ident(value.clone());
                value
            }
        }
    } else if let Some(default) = default {
        quote! {
            #field_name: #loaded_params_var.get(#var_name)
                .and_then(|v| v.parse::<#field_type>().ok())
                .unwrap_or_else(|| #default.parse::<#field_type>().unwrap())

        }
    } else {
        quote! {
            #field_name: #loaded_params_var.get(#var_name)
                .and_then(|v| v.parse::<#field_type>().ok())
                .unwrap_or_default()
        }
    }
}

fn handle_field_assigns<'a>(
    fields: &'a Fields,
    env_args: &'a StructEnvArgs,
    loaded_params_var: &'a proc_macro2::Ident,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    fields
        .iter()
        .map(move |field| handle_field_assign(env_args, field, loaded_params_var))
}

/// Extracts the variable name from the field attributes, looking for a specific attribute name.
/// If the attribute is not found, it returns `None`.
///
/// # Arguments
/// * `field` - The field from which to extract the variable name.
/// * `field_name` - The name of the attribute to look for.
/// # Returns
/// * `Option<String>` - The variable name if found, otherwise `None`.
///
/// # Example
/// ```rust,ignore
/// let field = ...; // Some syn::Field
/// let var_name = get_var_name(&field, "from");
/// if let Some(name) = var_name {
///     println!("Found variable name: {}", name);
/// } else {
///     println!("Variable name not found.");
/// }
/// ```
fn get_var_name(field: &Field, field_name: &'static str) -> Option<String> {
    for attr in &field.attrs {
        if attr.path().is_ident("conf") {
            if let Meta::List(meta_list) = &attr.meta {
                if let Ok(args) =
                    meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    for meta in args {
                        if let Meta::NameValue(name_value) = meta {
                            if name_value.path.is_ident(field_name) {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(lit_str),
                                    ..
                                }) = &name_value.value
                                {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
                let mut result = None;
                let _ = meta_list.parse_nested_meta(|meta| {
                    if meta.path.is_ident(field_name) {
                        if let Ok(value) = meta.value()?.parse::<syn::LitStr>() {
                            result = Some(value.value());
                        }
                    }
                    Ok(())
                });
                if result.is_some() {
                    return result;
                }
            }
        }
    }
    None
}
