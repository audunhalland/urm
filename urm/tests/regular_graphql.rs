use async_graphql::*;
use async_trait::*;

/*
struct Test;

#[async_graphql::Object]
impl Test {
    async fn rec(&self) -> Test {
        Test
    }
}
*/

use async_graphql::*;
use async_trait::*;
struct Test;
impl Test {
    async fn rec(&self, _: &async_graphql::Context<'_>) -> async_graphql::Result<Test> {
        {
            ::std::result::Result::Ok(
                async move {
                    let value: Test = { Test };
                    value
                }
                .await,
            )
        }
    }
}
#[allow(clippy::all, clippy::pedantic)]
impl async_graphql::Type for Test {
    fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
        ::std::borrow::Cow::Borrowed("Test")
    }
    fn create_type_info(registry: &mut async_graphql::registry::Registry) -> ::std::string::String {
        let ty =
            registry.create_type::<Self, _>(|registry| async_graphql::registry::MetaType::Object {
                name: ::std::borrow::ToOwned::to_owned("Test"),
                description: ::std::option::Option::None,
                fields: {
                    let mut fields = async_graphql::indexmap::IndexMap::new();
                    fields.insert(
                        ::std::borrow::ToOwned::to_owned("rec"),
                        async_graphql::registry::MetaField {
                            name: ::std::borrow::ToOwned::to_owned("rec"),
                            description: ::std::option::Option::None,
                            args: {
                                let mut args = async_graphql::indexmap::IndexMap::new();
                                args
                            },
                            ty: <Test as async_graphql::Type>::create_type_info(registry),
                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            external: false,
                            provides: ::std::option::Option::None,
                            requires: ::std::option::Option::None,
                            visible: ::std::option::Option::None,
                            compute_complexity: ::std::option::Option::None,
                        },
                    );
                    fields
                },
                cache_control: async_graphql::CacheControl {
                    public: true,
                    max_age: 0usize,
                },
                extends: false,
                keys: ::std::option::Option::None,
                visible: ::std::option::Option::None,
            });
        ty
    }
}
#[allow(clippy::all, clippy::pedantic, clippy::suspicious_else_formatting)]
#[allow(unused_braces, unused_variables, unused_parens, unused_mut)]
impl async_graphql::resolver_utils::ContainerType for Test {
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn resolve_field<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        ctx: &'life1 async_graphql::Context<'life2>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                async_graphql::ServerResult<::std::option::Option<async_graphql::Value>>,
            > {
                return __ret;
            }
            let __self = self;
            let ctx = ctx;
            let __ret: async_graphql::ServerResult<::std::option::Option<async_graphql::Value>> = {
                if ctx.item.node.name.node == "rec" {
                    let f = async move {
                        {
                            let res = __self.rec(ctx).await;
                            res.map_err(|err| {
                                ::std::convert::Into::<async_graphql::Error>::into(err)
                                    .into_server_error(ctx.item.pos)
                            })
                        }
                    };
                    let obj = f.await.map_err(|err| ctx.set_error_path(err))?;
                    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
                    return async_graphql::OutputType::resolve(&obj, &ctx_obj, ctx.item)
                        .await
                        .map(::std::option::Option::Some);
                }
                ::std::result::Result::Ok(::std::option::Option::None)
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn find_entity<'life0, 'life1, 'life2, 'life3, 'async_trait>(
        &'life0 self,
        ctx: &'life1 async_graphql::Context<'life2>,
        params: &'life3 async_graphql::Value,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                    Output = async_graphql::ServerResult<
                        ::std::option::Option<async_graphql::Value>,
                    >,
                > + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        'life3: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                async_graphql::ServerResult<::std::option::Option<async_graphql::Value>>,
            > {
                return __ret;
            }
            let __self = self;
            let ctx = ctx;
            let params = params;
            let __ret: async_graphql::ServerResult<::std::option::Option<async_graphql::Value>> = {
                let params = match params {
                    async_graphql::Value::Object(params) => params,
                    _ => return ::std::result::Result::Ok(::std::option::Option::None),
                };
                let typename =
                    if let ::std::option::Option::Some(async_graphql::Value::String(typename)) =
                        params.get("__typename")
                    {
                        typename
                    } else {
                        return ::std::result::Result::Err(async_graphql::ServerError::new(
                            r#""__typename" must be an existing string."#,
                            ::std::option::Option::Some(ctx.item.pos),
                        ));
                    };
                ::std::result::Result::Ok(::std::option::Option::None)
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
#[allow(clippy::all, clippy::pedantic)]
impl async_graphql::OutputType for Test {
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn resolve<'life0, 'life1, 'life2, 'life3, 'async_trait>(
        &'life0 self,
        ctx: &'life1 async_graphql::ContextSelectionSet<'life2>,
        _field: &'life3 async_graphql::Positioned<async_graphql::parser::types::Field>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = async_graphql::ServerResult<async_graphql::Value>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        'life3: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<async_graphql::ServerResult<async_graphql::Value>>
            {
                return __ret;
            }
            let __self = self;
            let ctx = ctx;
            let _field = _field;
            let __ret: async_graphql::ServerResult<async_graphql::Value> =
                { async_graphql::resolver_utils::resolve_container(ctx, __self).await };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
impl async_graphql::ObjectType for Test {}
