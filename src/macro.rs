macro_rules! expand {
    ($macro:ident, $letter:ident) => {
        //$macro!($letter);
    };
    ($macro:ident, $letter:ident, $($tail:ident),*) => {
        $macro!($letter, $($tail),*);
        expand!($macro, $($tail),*);
    };
}

macro_rules! impl_for_tuples {
    ($macro:ident) => {
        expand!($macro, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
    };
}

macro_rules! count {
    () => {0usize};
    ($head:tt $($tail:tt)*) => {1usize + count!($($tail)*)};
}

#[cfg(all(test, feature = "parallel"))]
macro_rules! wrap_helper {
    ($var:ident, $var_type:ident, $borrow:expr) => {
        <Ref<$var_type> as WrappableSingle<&$var_type, ()>>::wrap(&mut &$var, &mut $borrow)
    };
    (mut $var:ident, $var_type:ident, $borrow:expr) => {
        <Mut<$var_type> as WrappableSingle<&mut $var_type, ()>>::wrap(&mut &mut $var, &mut $borrow)
    };
}
