#[policy_proc_macro::policy_carrying]
struct Foo {
    #[allow(
        attribute_list => ["foo", "bar"];
        scheme => [];
    )]
    ok: i32,
}
