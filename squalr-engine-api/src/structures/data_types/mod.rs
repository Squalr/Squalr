/// Re-export the domain crate data types so API consumers share one source of truth.
pub use squalr_engine_domain::structures::data_types::*;

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use super::{built_in_types::u8::data_type_u8::DataTypeU8 as ApiDataTypeU8, data_type_ref::DataTypeRef as ApiDataTypeRef};
    use squalr_engine_domain::structures::data_types::{
        built_in_types::u8::data_type_u8::DataTypeU8 as DomainDataTypeU8, data_type_ref::DataTypeRef as DomainDataTypeRef,
    };

    #[test]
    fn api_reexports_domain_data_types() {
        assert_eq!(TypeId::of::<ApiDataTypeRef>(), TypeId::of::<DomainDataTypeRef>());
        assert_eq!(TypeId::of::<ApiDataTypeU8>(), TypeId::of::<DomainDataTypeU8>());
    }
}
