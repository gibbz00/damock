#[cfg_attr(test, derive(damock::Mock))]
enum Foo {
    #[cfg_attr(test, mock)]
    Bar,
}

fn main() {
    <Foo as damock::Mock>::mock();
}
