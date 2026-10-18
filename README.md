# Learning by example

This is basically an excerpt from Kopf classes (a framework for Python to make Kubernetes operators), but implemented in Rust.

This is nothing more than me learning Rust and its memory model after reading the "The Rust Programming Language" book (aka "the book").

Sample usage:

```rust
use example1::{Resource, Selector};

let r1 = Resource::builder().plural("pods").group("").version("v1").build();
let r2 = Resource::builder().plural("kopfexamples").group("kopf.dev").version("v1").build();

let s1 = Selector::parse("pods.v1");
let onlyv1 = s1.select([&r1, &r2]);

let s2 = Selector::parse(("kopf.dev", "kopfexamples"));
let kexes = s2.select([&r1, &r2]);

let s3 = Selector::builder().group("kopf.dev").any_name("kopfexamples").build();
let kexes = s2.select([&r1, &r2]);
```

There is some simulation of variadic args (1-3 strings), both to the main class of the selector and to its builder, so as the builder pattern implemented (hopefully idiomatically).

And a lot of tests (but not as thorough as I usually have).

To be continued…