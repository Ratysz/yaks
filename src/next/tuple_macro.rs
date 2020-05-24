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
