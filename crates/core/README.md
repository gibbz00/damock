# Damock - Composable Data Mocking

[![ci_status](https://img.shields.io/github/actions/workflow/status/gibbz00/damock/ci.yaml?style=for-the-badge)](https://github.com/gibbz00/damock/actions/workflows/ci.yaml)
[![codecov](https://img.shields.io/codecov/c/gh/gibbz00/damock?token=5lHDbjv0AQ&style=for-the-badge)](https://codecov.io/gh/gibbz00/damock)
[![license](https://img.shields.io/github/license/gibbz00/damock.svg?style=for-the-badge)](https://github.com/gibbz00/damock/blob/main/LICENSE.md)
[![crates_io](https://img.shields.io/crates/v/damock.svg?style=for-the-badge)](https://crates.io/crates/damock)
[![docs_rs](https://img.shields.io/docsrs/damock/latest.svg?style=for-the-badge)](https://docs.rs/damock)

## Derivable and conditionally compiled data mocking

```rust
use damock::Mock;

#[cfg_attr(test, derive(Mock))]
struct Foo {
    bar: Bar,
    // Use `Default::default` rather than `Mock::mock`
    #[cfg_attr(test, mock_default)]
    baz: u8
}

#[cfg_attr(test, derive(Mock))]
enum Bar {
    // Define which enum variant shall be used for mocking
    #[cfg_attr(test, mock)]
    A,
    B,
}
```

The derive of `Foo` expands into:

```rust
#[cfg(test)]
impl Mock for Foo {
    fn mock() -> Self {
        Self {
            bar: Mock::mock(),
            baz: Default::default(),
        }
    }
}
```

Derived mock implementations behind `cfg_attr` are always conditionally compiled
under the same predicate.

The `test` compiler configuration may be overridden to something else like so:

```rust
#[cfg_attr(feature = "mocks", derive(Mock))]
struct Foo;
```

No conditional compilation is applied when no `cfg_attr` is used, e.g. `#[derive(Mock)]`.

## Toy application

```no_compile
#[test]
fn computes_data() {
  let actual = compute(DataInput::mock());
  assert_eq!(DataOutput::mock(), actual);
}
```
