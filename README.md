# diplomat-tool error masking reproduction

Minimal reproduction for an error-reporting bug in `diplomat-tool`.

Verified against:

- `diplomat-tool 0.16.0`
- `diplomat-tool 0.17.0` from upstream `main` at commit `71628f0` (2026-11-15)

## 概要

使用 `diplomat-tool` 生成绑定时，若入口 `lib.rs` 中的内联 `#[diplomat::bridge] mod ffi { ... }` 内出现类型解析错误（例如方法返回 diplomat 不识别的 `String`），工具不会报告真实的解析错误，而是直接 panic：

```
Could not read source file src: Os { code: 21, kind: IsADirectory, message: "Is a directory" }
```

该 panic 来自错误报告代码本身，掩盖了底层真正的错误（如 `Could not resolve String`）。

## 复现

本工程无需 `Cargo.toml` 或 config 文件，`diplomat-tool` 直接解析源码。

源码见 [`src/lib.rs`](src/lib.rs)：

```rust
#[diplomat::bridge]
pub mod ffi {
    #[diplomat::opaque]
    pub struct Foo;

    impl Foo {
        // `String` 不是 diplomat 识别的类型，解析为自定义类型时会失败
        pub fn name(&self) -> String {
            String::new()
        }
    }
}
```

在工程根目录执行：

```bash
rustup toolchain install 1.97.1
cargo install diplomat-tool --version 0.16.0
RUST_BACKTRACE=1 diplomat-tool kotlin out/ -e src/lib.rs
```

## 实际输出与堆栈

```
thread 'main' (5) panicked at /home/gxrs/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/diplomat_core-0.16.0/src/ast/logging.rs:102:21:
Could not read source file src: Os { code: 21, kind: IsADirectory, message: "Is a directory" }
stack backtrace:
   0: __rustc::rust_begin_unwind
   1: core::panicking::panic_fmt
   2: diplomat_core::ast::logging::create_report
   3: diplomat_core::ast::logging::create_simple_report
   4: <diplomat_core::ast::types::PathType>::resolve_with_path
   5: <diplomat_core::ast::types::PathType>::resolve
   6: <diplomat_core::hir::lowering::LoweringContext>::lower_out_type::<diplomat_core::hir::elision::ReturnLifetimeLowerer>
   7: <diplomat_core::hir::lowering::LoweringContext>::lower_return_type
   8: <diplomat_core::hir::lowering::LoweringContext>::lower_all_methods
   9: <diplomat_core::hir::lowering::LoweringContext>::lower_opaque
  10: <diplomat_core::hir::lowering::LoweringContext>::lower_all_opaques::<smallvec::IntoIter<[diplomat_core::hir::lowering::ItemAndInfo<diplomat_core::ast::opaque::OpaqueType>; 16]>>
  11: <diplomat_core::hir::type_context::TypeContext>::from_syn::<diplomat_core::hir::attrs::BasicAttributeValidator>
  12: diplomat_tool::gen
  13: diplomat_tool::main
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
```

## 说明

panic 发生在错误报告路径中：`create_report` 尝试读取 span 对应的源文件以生成错误上下文，但该 span 位置被设为入口文件的父目录（此处为 `src`，一个目录），`read_to_string` 失败导致 panic。因此底层真正的类型解析错误（如 `Could not resolve String`）未被显示。

上游 `diplomat-tool` 将入口文件的父目录作为 `SpanLocation::FilePath` 传入，而不是传入入口文件本身。随后错误报告代码尝试把该目录当作源码文件读取，从而触发 panic。
