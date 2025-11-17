use proc_macro2::Literal;
use quote::quote;

#[proc_macro_derive(InputMapped)]
pub fn derive_input_mapped(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use quote::{format_ident, quote};
    use venial::{Item, Struct};

    let Item::Struct(item) = venial::parse_item(input.into()).unwrap() else {
        return quote! { compile_error!("expected a struct") }.into();
    };

    if item.generic_params.is_some() {
        return quote! { compile_error!("generic paramameters are not supported") }.into();
    }

    let Struct {
        vis_marker, name, ..
    } = &item;

    let field_names = item.field_tokens().into_iter().collect::<Vec<_>>();
    let field_types = item.field_types().into_iter().collect::<Vec<_>>();
    let field_indices = (0..field_names.len())
        .map(|n| Literal::usize_unsuffixed(n))
        .collect::<Vec<_>>();

    let bindings_name = format_ident!("{name}Bindings");

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        #vis_marker struct #bindings_name {
            #(#field_names: <#field_types as lib_input::InputMapped>::Bindings),*
        }

        impl lib_input::InputMapped for #name {
            type Bindings = #bindings_name;
            type MapperState = (#(<#field_types as lib_input::InputMapped>::MapperState,)*);

            fn new_mapper(bindings: &Self::Bindings) -> Self::MapperState {
                (#(<#field_types as lib_input::InputMapped>::new_mapper(&bindings.#field_names),)*)
            }

            fn mapper_event(
                handler: &mut Self::MapperState,
                event: lib_window::DeviceEvent<'_>,
                ctx: &lib_input::MapperContext,
            ) {
                #(<#field_types as lib_input::InputMapped>::mapper_event(&mut handler.#field_indices, event, ctx);)*
            }

            fn map(handler: &mut Self::MapperState) -> Self {
                Self {
                    #(#field_names: <#field_types as lib_input::InputMapped>::map(&mut handler.#field_indices),)*
                }
            }
        }
    }
    .into()
}
