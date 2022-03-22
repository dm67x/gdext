use miniserde::{json, Deserialize};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::Path;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// JSON models

#[derive(Deserialize)]
struct ExtensionApi {
    builtin_class_sizes: Vec<ClassSizes>,
    global_enums: Vec<GlobalEnum>,
}

#[derive(Deserialize)]
struct ClassSizes {
    build_configuration: String,
    sizes: Vec<ClassSize>,
}

#[derive(Deserialize)]
struct ClassSize {
    name: String,
    size: usize,
}

#[derive(Deserialize)]
struct GlobalEnum {
    name: String,
    values: Vec<EnumValue>,
}

#[derive(Deserialize)]
struct EnumValue {
    name: String,
    value: i32,
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Implementation

struct Tokens {
    opaque_types: Vec<TokenStream>,
    variant_enumerators: Vec<TokenStream>,
    from_variant_fns: Vec<TokenStream>,
}

pub struct ApiParser {}

impl ApiParser {
    pub fn generate_file(gen_path: &Path) {
        let tokens = Self::load_extension_api();
        let Tokens {
            opaque_types,
            variant_enumerators,
            from_variant_fns,
        } = tokens;

        let tokens = quote! {
            pub mod types {
                #(#opaque_types)*
            }

            pub mod variant_conv {
                #(#from_variant_fns)*
            }

            pub enum VariantType {
                #(#variant_enumerators),*
            }
        };

        let string = tokens.to_string();

        let _ = std::fs::create_dir(gen_path);
        let out_path = gen_path.join("extensions.rs");
        std::fs::write(&out_path, string).expect("failed to write extension file");
        Self::format_file_if_needed(&out_path);
    }

    fn load_extension_api() -> Tokens {
        let build_config = "float_64"; // TODO infer this

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/input/extension_api.json");
        let json = std::fs::read_to_string(path).expect(&format!("failed to open file {:?}", path));
        let model: ExtensionApi = json::from_str(&json).expect("failed to deserialize JSON");

        let mut opaque_types = vec![];
        let mut variant_enumerators = vec![];
        let mut from_variant_fns = vec![];

        for class in &model.builtin_class_sizes {
            if &class.build_configuration == build_config {
                for ClassSize { name, size } in &class.sizes {
                    opaque_types.push(Self::quote_opaque_type(name, *size));
                }

                break;
            }
        }

        for enum_ in &model.global_enums {
            if &enum_.name == "Variant.Type" {
                for ty in &enum_.values {
                    let name = ty
                        .name
                        .strip_prefix("TYPE_")
                        .expect("Enum name begins with 'TYPE_'");
                    let value = ty.value;

                    if name == "MAX" {
                        continue;
                    }
                    variant_enumerators.push(Self::quote_enumerator(name, value));
                    from_variant_fns.push(Self::quote_from_variant(name));
                }

                break;
            }
        }

        Tokens {
            opaque_types,
            variant_enumerators,
            from_variant_fns,
        }
    }

    fn quote_enumerator(name: &str, value: i32) -> TokenStream {
        let enumerator = format_ident!("{}", name);
        let value = proc_macro2::Literal::i32_unsuffixed(value);

        quote! {
           #enumerator = #value
        }
    }

    fn quote_opaque_type(name: &str, size: usize) -> TokenStream {
        // Capitalize: "int" -> "Int"
        let (first, rest) = name.split_at(1);
        let ident = format_ident!("Opaque{}{}", first.to_uppercase(), rest);
        quote! {
            pub type #ident = [u8; #size];
        }
    }

    fn quote_from_variant(upper_name: &str) -> TokenStream {
        let fn_name = format_ident!("variant_from_{}", upper_name.to_lowercase());
        let variant_type =
            format_ident!("GDNativeVariantType_GDNATIVE_VARIANT_TYPE_{}", upper_name);

        // FIXME can't directly save as static, load dynamically
        quote! {
            static #fn_name: unsafe extern "C" fn(crate::GDNativeVariantPtr, crate::GDNativeTypePtr) = unsafe {
                let ctor_fn = crate::get_interface().get_variant_from_type_constructor.unwrap_unchecked();
                ctor_fn(crate:: #variant_type).unwrap()
            };
        }
    }

    //#[cfg(feature = "formatted")]
    fn format_file_if_needed(output_rs: &Path) {
        print!(
            "Formatting generated file: {}... ",
            output_rs
                .canonicalize()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
        );

        let output = std::process::Command::new("rustup")
            .arg("run")
            .arg("stable")
            .arg("rustfmt")
            .arg("--edition=2021")
            .arg(output_rs)
            .output();

        match output {
            Ok(_) => println!("Done."),
            Err(err) => {
                println!("Failed.");
                println!("Error: {}", err);
            }
        }
    }
}