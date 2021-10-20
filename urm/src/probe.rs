use std::future::Future;

pub fn probe_container<T: async_graphql::ContainerType>(
    container: &T,
    ctx: &async_graphql::context::Context<'_>,
) {
    let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
    probe_container_selection_set(container, &ctx_obj);
}

fn probe_container_selection_set<T: async_graphql::ContainerType>(
    container: &T,
    ctx: &async_graphql::ContextSelectionSet,
) {
    let waker = noop_waker::noop_waker();
    let mut fut_ctx = std::task::Context::from_waker(&waker);

    for selection in &ctx.item.node.items {
        if ctx.is_skip(&selection.node.directives()).unwrap_or(true) {
            continue;
        }

        match &selection.node {
            async_graphql::parser::types::Selection::Field(field) => {
                if field.node.name.node == "__typename" {
                    continue;
                }

                if ctx.is_ifdef(&field.node.directives) {
                    if let Some(async_graphql::registry::MetaType::Object { fields, .. }) =
                        ctx.schema_env.registry.types.get(T::type_name().as_ref())
                    {
                        if !fields.contains_key(field.node.name.node.as_str()) {
                            continue;
                        }
                    }
                }

                let future = {
                    let ctx = ctx.clone();

                    async move {
                        let ctx_field = ctx.with_field(field);
                        container.resolve_field(&ctx_field).await.unwrap();
                    }
                };
                futures_util::pin_mut!(future);

                match future.poll(&mut fut_ctx) {
                    std::task::Poll::Ready(_) => {}
                    std::task::Poll::Pending => {}
                }
            }
            _ => {}
        }
    }
}
