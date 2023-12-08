extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use walkdir::WalkDir;
use syn::{self, Ident};
use proc_macro2::Span;


// Function to convert CamelCase to snake_case and then to an Ident
fn camel_to_snake(name: &str) -> Ident {
    let mut snake_case = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            snake_case.push('_');
        }
        snake_case.push(ch.to_ascii_lowercase());
    }
    Ident::new(&snake_case, Span::call_site())
}

#[proc_macro]
pub fn auto_register_tools(_item: TokenStream) -> TokenStream {
    let struct_re = Regex::new(r"pub struct\s+(\w+)").unwrap();
    let mut registrations = Vec::new();

    // Iterate over files in the `tools/` directory
    for entry in WalkDir::new("src/tools").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let content = std::fs::read_to_string(entry.path()).expect("Unable to read file");

            // Find all struct declarations
            for cap in struct_re.captures_iter(&content) {
                let struct_name = &cap[1];
                let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
                let module_name = camel_to_snake(&struct_ident.to_string());

                // Generate imports and registration calls
                registrations.push(quote! {
                    use crate::tools::#module_name::#struct_ident;

                    registry.register(#struct_ident);
                });
            }
        }
    }

    // Combine everything into a single output
    let expanded = quote! {
        pub fn register_tools(registry: &mut ToolRegistry) {
            #( #registrations )*
        }
    };

    expanded.into()
}
