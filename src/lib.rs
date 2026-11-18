#[diplomat::bridge]
pub mod ffi {
    #[diplomat::opaque]
    pub struct Foo;

    impl Foo {
        // `String` is not a recognized diplomat type; resolving it as a
        // custom type fails and triggers an error report.
        pub fn name(&self) -> String {
            String::new()
        }
    }
}
