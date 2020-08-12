extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate uuid;

use proc_macro::TokenStream;

use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn uuid(ts: TokenStream) -> TokenStream {
    let s = parse_macro_input!(ts as LitStr);

    // TODO: better error handling
    let uuid = uuid::Uuid::parse_str(&s.value()).unwrap();
    let bytes: [u8; 16] = *uuid.as_bytes();

    let b0 = bytes[0];
    let b1 = bytes[1];
    let b2 = bytes[2];
    let b3 = bytes[3];
    let b4 = bytes[4];
    let b5 = bytes[5];
    let b6 = bytes[6];
    let b7 = bytes[7];
    let b8 = bytes[8];
    let b9 = bytes[9];
    let b10 = bytes[10];
    let b11 = bytes[11];
    let b12 = bytes[12];
    let b13 = bytes[13];
    let b14 = bytes[14];
    let b15 = bytes[15];

    let t = quote! {
        ::uuid::Uuid::from_bytes(
            [
                #b0,
                #b1,
                #b2,
                #b3,
                #b4,
                #b5,
                #b6,
                #b7,
                #b8,
                #b9,
                #b10,
                #b11,
                #b12,
                #b13,
                #b14,
                #b15,
            ]
        )
    };

    t.into()
}

#[proc_macro]
pub fn uuid_u128(ts: TokenStream) -> TokenStream {
    let s = parse_macro_input!(ts as LitStr);

    // TODO: better error handling
    let uuid = uuid::Uuid::parse_str(&s.value()).unwrap();
    let uuid_u128 = uuid.as_u128();

    let t = quote! {
        #uuid_u128
    };

    t.into()
}
