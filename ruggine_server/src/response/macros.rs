#[macro_export]
macro_rules! api_success_response_alias {
    ($alias:ident, $inner:ty) => {
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
        pub struct $alias {
            #[schema(inline)]
            pub data: $inner,
        }
    };
}