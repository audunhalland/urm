/// Something that is never ready
pub async fn never<T>() -> T {
    let fut = Never::<T>(std::marker::PhantomData);
    fut.await
}

struct Never<T>(std::marker::PhantomData<T>);

impl<T> std::future::Future for Never<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Pending
    }
}
