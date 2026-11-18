# diplomat-tool error masking reproduction
## diplomat-tool 错误掩盖复现

Minimal reproduction for an error-reporting bug in `diplomat-tool`.<br>
`diplomat-tool` 错误报告问题的最小复现。

Verified with `diplomat-tool 0.16.0` and upstream `main@71628f0` (`0.17.0`, 2026-11-15).<br>
已在 `diplomat-tool 0.16.0` 和上游 `main@71628f0`（`0.17.0`，2026-11-15）上验证。

## Overview
## 概要

When `diplomat-tool` generates bindings and a type-resolution error occurs inside an inline `#[diplomat::bridge] mod ffi { ... }` module, the tool may panic instead of reporting the real error. This reproduction uses a method that returns `String`, which Diplomat does not recognize as a supported type.<br>
使用 `diplomat-tool` 生成绑定时，如果内联 `#[diplomat::bridge] mod ffi { ... }` 模块内出现类型解析错误，工具可能直接 panic，而不是报告真实错误。本复现使用一个返回 `String` 的方法，而 Diplomat 不将该类型识别为支持的类型。

Actual output:<br>
实际输出：

```text
Could not read source file src: Os { code: 21, kind: IsADirectory, message: "Is a directory" }
```

This panic comes from the error-reporting code and masks the underlying error, such as `Could not resolve String`.<br>
该 panic 来自错误报告代码本身，掩盖了底层真正的错误，例如 `Could not resolve String`。

## Reproduction
## 复现

This reproduction does not require a `Cargo.toml` or config file. `diplomat-tool` parses the source file directly.<br>
本工程不需要 `Cargo.toml` 或配置文件，`diplomat-tool` 直接解析源码。

The source is in [`src/lib.rs`](src/lib.rs):<br>
源码见 [`src/lib.rs`](src/lib.rs)：

```rust
#[diplomat::bridge]
pub mod ffi {
    #[diplomat::opaque]
    pub struct Foo;

    impl Foo {
        // `String` is not a recognized Diplomat type.
        // `String` 不是 Diplomat 识别的类型。
        pub fn name(&self) -> String {
            String::new()
        }
    }
}
```

Run the following commands from the repository root:<br>
在仓库根目录执行：

```bash
rustup toolchain install 1.97.1
cargo install diplomat-tool --version 0.16.0
RUST_BACKTRACE=1 diplomat-tool kotlin out/ -e src/lib.rs
```

## Actual output and backtrace
## 实际输出与堆栈

```text
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

## Root cause
## 原因

The panic occurs while reporting an earlier error. `create_report` tries to read the source file associated with a span, but that span points to the entry file's parent directory (`src`) rather than the entry file (`src/lib.rs`). Calling `read_to_string` on the directory fails and panics, so the original type-resolution error is never displayed.<br>
panic 发生在报告更早错误的过程中。`create_report` 尝试读取 span 对应的源文件以生成错误上下文，但该 span 指向入口文件的父目录 `src`，而不是入口文件 `src/lib.rs`。对目录调用 `read_to_string` 失败后触发 panic，因此底层真正的类型解析错误没有被显示。

Upstream `diplomat-tool` passes the entry file's parent directory as `SpanLocation::FilePath` instead of passing the entry file itself. The error-reporting code then attempts to read that directory as a source file.<br>
上游 `diplomat-tool` 将入口文件的父目录作为 `SpanLocation::FilePath` 传入，而不是传入入口文件本身。随后错误报告代码尝试把该目录当作源码文件读取，从而触发 panic。
