use yaks::{CantInsertSystem, Executor, ModQueuePool, Resources, System, World};

struct Res1(usize);

struct Res2(f32);

struct Comp1(usize);

struct Comp2(f32);

struct Comp3(&'static str);

fn setup() -> (World, Resources, ModQueuePool) {
    let mut world = World::new();
    world.spawn((Comp1(1), Comp2(0.0)));
    world.spawn((Comp1(0), Comp2(1.0)));
    world.spawn((Comp1(1), Comp2(2.0), Comp3("one")));
    world.spawn((Comp1(2), Comp2(1.0), Comp3("two")));
    world.spawn((Comp1(1), Comp3("one")));
    world.spawn((Comp2(1.0), Comp3("two")));
    let mut resources = Resources::new();
    resources.insert(Res1(0));
    resources.insert(Res2(0.0));
    (world, resources, ModQueuePool::new())
}

#[test]
fn single_no_handle() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<()>::builder()
        .system(
            System::builder()
                .resources::<&mut Res1>()
                .build(move |_, mut resource, _| {
                    resource.0 += 1;
                }),
        )
        .build();
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
}

#[test]
fn non_unique_system_handle() {
    let mut executor = Executor::<usize>::new();
    let option = executor
        .insert_with_handle(System::builder().build(|_, _, _| {}), 0)
        .unwrap();
    assert!(option.is_none());
    let option = executor
        .insert_with_handle(System::builder().build(|_, _, _| {}), 0)
        .unwrap();
    assert!(option.is_some());
}

#[test]
fn single_handle() {
    let (world, resources, mod_queues) = setup();
    let mut executor = Executor::<usize>::builder()
        .system_with_handle(
            System::builder()
                .resources::<&mut Res1>()
                .build(move |_, mut resource, _| {
                    resource.0 += 1;
                }),
            0,
        )
        .build();
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert!(executor.is_active(&0).unwrap());
    assert!(executor.set_active(&0, false).is_ok());
    assert!(!executor.is_active(&0).unwrap());
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 1);
    assert!(executor.set_active(&0, true).is_ok());
    executor.run(&world, &resources, &mod_queues);
    assert_eq!(resources.get::<Res1>().unwrap().0, 2);
    assert!(executor.is_active(&2).is_err())
}

#[test]
fn invalid_dependencies() {
    let mut executor = Executor::<usize>::new();
    let result =
        executor.insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 0, vec![1]);
    assert!(result.is_err());
    if let Err(error) = result {
        assert_ne!(error, CantInsertSystem::CyclicDependency);
    }
}

#[test]
fn cyclic_dependency_1() {
    let mut executor = Executor::<usize>::new();
    assert!(executor
        .insert_with_handle(System::builder().build(|_, _, _| {}), 0,)
        .is_ok());
    let result =
        executor.insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 0, vec![0]);
    assert!(result.is_err());
    if let Err(error) = result {
        assert_eq!(error, CantInsertSystem::CyclicDependency);
    }
}

#[test]
fn cyclic_dependency_2() {
    let mut executor = Executor::<usize>::new();
    assert!(executor
        .insert_with_handle(System::builder().build(|_, _, _| {}), 0,)
        .is_ok());
    assert!(executor
        .insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 1, vec![0],)
        .is_ok());
    let result =
        executor.insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 0, vec![1]);
    assert!(result.is_err());
    if let Err(error) = result {
        assert_eq!(error, CantInsertSystem::CyclicDependency);
    }
}

#[test]
fn cyclic_dependency_3() {
    let mut executor = Executor::<usize>::new();
    assert!(executor
        .insert_with_handle(System::builder().build(|_, _, _| {}), 0)
        .is_ok());
    assert!(executor
        .insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 1, vec![0],)
        .is_ok());
    assert!(executor
        .insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 2, vec![1],)
        .is_ok());
    let result =
        executor.insert_with_handle_and_deps(System::builder().build(|_, _, _| {}), 0, vec![2]);
    assert!(result.is_err());
    if let Err(error) = result {
        assert_eq!(error, CantInsertSystem::CyclicDependency);
    }
}
