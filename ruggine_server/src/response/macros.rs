#[macro_export]
macro_rules! api_success_response_alias {
    ($alias:ident, $inner:ty) => {
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
        pub struct $alias {
            #[schema(inline)]
            pub data: $inner,
        }

        impl $alias {
            pub fn data(&self) -> &$inner {
                &self.data
            }
        }
    };
}

#[macro_export]
macro_rules! paginated_response_alias {
    ($alias:ident, $data_ty:ty, $pagination_ty:ty) => {
        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
        pub struct $alias {
            #[schema(inline)]
            pub data: Vec<$data_ty>,
            #[schema(inline)]
            pub pagination: $pagination_ty,
        }

        impl $alias {
            pub fn new(data: Vec<$data_ty>, pagination: $pagination_ty) -> Self {
                Self { data, pagination }
            }

            pub fn data(&self) -> &Vec<$data_ty> {
                &self.data
            }

            pub fn pagination(&self) -> &$pagination_ty {
                &self.pagination
            }
        }
    };
}

#[macro_export]
macro_rules! pagination_metadata_alias {
    ($alias:ident, $cursor_ty:ty) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
        pub struct $alias {
            /// Whether there are more items available
            #[schema(example = true)]
            pub has_more: bool,

            /// Cursor for the next page (timestamp or ID)
            #[schema(example = "2025-08-12T10:30:00Z")]
            pub next_cursor: Option<$cursor_ty>,

            /// Number of items in this page
            #[schema(example = 20)]
            pub page_size: usize,

            /// Total count (optional)
            #[schema(example = 150)]
            pub total_count: Option<usize>,
        }

        impl $alias {
            pub fn new(has_more: bool, next_cursor: Option<$cursor_ty>, page_size: usize, total_count: Option<usize>) -> Self {
                Self {
                    has_more,
                    next_cursor,
                    page_size,
                    total_count,
                }
            }
        }
    };
}


