pub struct Foo;
pub struct Bar;

#[urm::table("foo")]
impl Foo {
    fn id(self) -> String;
    fn bar_id(self) -> String;

    #[foreign(Self(id) => Self(id))]
    fn self_to_self(self) -> Foo;

    #[foreign(Bar(id) => Bar(id))]
    fn bar_to_bar(self) -> Bar;
}

#[urm::table("bar")]
impl Bar {
    fn id(self) -> String;
    fn foo_id(self) -> String;
}

fn main() {}
