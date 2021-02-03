use quote::quote;
use syn::{parse_macro_input, ItemTrait};

/// Generates an implementation of `Pointee` for trait objects.
#[proc_macro_attribute]
pub fn ptr_meta(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemTrait);

    let ident = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let result = quote! {
        #input

        const _: () = {
            use ptr_meta::{DynMetadata, Pointee};

            impl #impl_generics Pointee for dyn #ident #ty_generics #where_clause {
                type Metadata = DynMetadata<Self>;
            }
        };
    };

    proc_macro::TokenStream::from(result)
}
