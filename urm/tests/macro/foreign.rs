pub struct Foo;
pub struct Bar;

#[urm::table("foo")]
impl Foo {
    fn id() -> String;
    fn bar_id() -> String;

    #[foreign(Self(id) => Self(id))]
    fn self_to_self() -> Foo;

    #[foreign(Bar(id) => Bar(id))]
    fn bar_to_bar() -> Bar;
}

#[urm::table("bar")]
impl Bar {
    fn id() -> String;
    fn foo_id() -> String;
}

fn main() {}
