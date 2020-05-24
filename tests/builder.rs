use yaks::{Executor, SystemContext};

fn dummy_system(_: SystemContext, _: (), _: ()) {}

#[test]
#[should_panic(expected = "system 0 already exists")]
fn duplicate_handle() {
    Executor::<()>::builder()
        .system_with_handle(dummy_system, 0)
        .system_with_handle(dummy_system, 0)
        .build();
}

#[test]
#[should_panic(expected = "system 0 already exists")]
fn duplicate_handle_with_deps() {
    Executor::<()>::builder()
        .system_with_handle(dummy_system, 0)
        .system_with_handle_and_deps(dummy_system, 0, vec![0])
        .build();
}

#[test]
#[should_panic(expected = "could not resolve dependencies of system 1: no system 2 found")]
fn invalid_dependency() {
    Executor::<()>::builder()
        .system_with_handle(dummy_system, 0)
        .system_with_handle_and_deps(dummy_system, 1, vec![2])
        .build();
}

#[test]
#[should_panic(
    expected = "could not resolve dependencies of a handle-less system: no system 1 found"
)]
fn invalid_dependency_no_handle() {
    Executor::<()>::builder()
        .system_with_handle(dummy_system, 0)
        .system_with_deps(dummy_system, vec![1])
        .build();
}

#[test]
#[should_panic(expected = "system 1 depends on itself")]
fn self_dependency() {
    Executor::<()>::builder()
        .system_with_handle(dummy_system, 0)
        .system_with_handle_and_deps(dummy_system, 1, vec![1])
        .build();
}
