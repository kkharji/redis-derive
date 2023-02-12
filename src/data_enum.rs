use super::{DeriveFromRedisArgs, DeriveToRedisArgs};

use quote::quote;
use syn::{DataEnum, Fields, Ident};

impl DeriveToRedisArgs for DataEnum {
    fn derive_to_redis(&self, type_ident: Ident) -> proc_macro::TokenStream {
        match self.variants {
            _ => todo!(),
        }
    }
}
