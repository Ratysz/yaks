# `yaks`
[![Latest Version]][crates.io]
[![Documentation]][docs.rs]
[![License]][license link]
[![CI]][CI link]

[Latest Version]: https://img.shields.io/crates/v/yaks.svg
[crates.io]: https://crates.io/crates/yaks
[Documentation]: https://docs.rs/yaks/badge.svg
[docs.rs]: https://docs.rs/yaks
[License]: https://img.shields.io/crates/l/yaks.svg
[license link]: https://github.com/Ratysz/yaks/blob/master/LICENSE.md
[CI]: https://github.com/Ratysz/yaks/workflows/CI/badge.svg?branch=master
[CI link]: https://github.com/Ratysz/yaks/actions?query=workflow%3ACI

`yaks` aims to be a minimalistic and performant framework for automatic
multithreading of [`hecs`] via [`rayon`].

The goals are, in no particular order:
- safety
- simplicity
- performance
- extensibility
- tight engineering
- minimal dependencies
- effortless concurrency

[`hecs`]: https://crates.io/crates/hecs
[`rayon`]: https://crates.io/crates/rayon

# Cargo features

- `parallel` - enabled by default; can be disabled to force `yaks` to work on a single thread.
Useful for writing the code once, and running it on platforms with or without threading.
- `resources-interop` - when enabled, allows `Executor::run()` to also
accept `Resources` struct from the [`resources`] crate in place of resources argument.

[`resources`]: https://crates.io/crates/resources

# Example

A more elaborate and annotated example can be found [here](examples/convoluted.rs).

```rust
use hecs::{With, Without, World};
use yaks::{Executor, QueryMarker};

fn main() {
    let mut world = World::new();
    let mut entities = 0u32;
    world.spawn_batch((0..100u32).map(|index| {
        entities += 1;
        (index,)
    }));
    world.spawn_batch((0..100u32).map(|index| {
        entities += 1;
        (index, index as f32)
    }));
    let mut increment = 5usize;
    let mut average = 0f32;
    let mut executor = Executor::<(u32, usize, f32)>::builder()
        .system_with_handle(
            |context, (entities, average): (&u32, &mut f32), query: QueryMarker<&f32>| {
                *average = 0.0;
                for (_entity, float) in context.query(query).iter() {
                    *average += *float;
                }
                *average /= *entities as f32;
            },
            "average",
        )
        .system_with_handle(
            |context, increment: &usize, query: QueryMarker<&mut u32>| {
                for (_entity, unsigned) in context.query(query).iter() {
                    *unsigned += *increment as u32
                }
            },
            "increment",
        )
        .system_with_deps(system_with_two_queries, vec!["increment", "average"])
        .build();
    executor.run(&world, (&mut entities, &mut increment, &mut average));
}

fn system_with_two_queries(
    context: yaks::SystemContext,
    (entities, average): (&u32, &f32),
    (with_f32, without_f32): (
        QueryMarker<With<f32, &mut u32>>,
        QueryMarker<Without<f32, &mut u32>>,
    ),
) {
    yaks::batch(
        &mut context.query(with_f32),
        entities / 8,
        |_entity, unsigned| {
            *unsigned += average.round() as u32;
        },
    );
    yaks::batch(
        &mut context.query(without_f32),
        entities / 8,
        |_entity, unsigned| {
            *unsigned *= average.round() as u32;
        },
    );
}
```
