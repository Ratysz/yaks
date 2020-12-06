macro_rules! expand {
    ($macro:ident, $letter:ident) => {
        $macro!($letter);
        $macro!();
    };
    ($macro:ident, $letter:ident, $($tail:ident),*) => {
        $macro!($letter, $($tail),*);
        expand!($macro, $($tail),*);
    };
}

macro_rules! impl_for_tuples {
    ($macro:ident) => {
        expand!($macro, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
        //expand!($macro, C, B, A);
    };
}

macro_rules! impl_for_res_and_query_tuples {
    ($macro:ident) => {
        impl_for_res_and_query_tuples!(@inner $macro, ());
        impl_for_res_and_query_tuples!(@inner $macro, (QA,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC, QD,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC, QD, QE,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC, QD, QE, QF,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC, QD, QE, QF, QG,));
        impl_for_res_and_query_tuples!(@inner $macro, (QA, QB, QC, QD, QE, QF, QG, QH,));
    };
    (@inner $macro:ident, ($($query:ident,)*)) => {
        $macro!((), ($($query,)*));
        $macro!(R, ($($query,)*));
        $macro!((RA,), ($($query,)*));
        $macro!((RA, RB,), ($($query,)*));
        $macro!((RA, RB, RC,), ($($query,)*));
        $macro!((RA, RB, RC, RD,), ($($query,)*));
        $macro!((RA, RB, RC, RD, RE,), ($($query,)*));
        $macro!((RA, RB, RC, RD, RE, RF,), ($($query,)*));
        $macro!((RA, RB, RC, RD, RE, RF, RG,), ($($query,)*));
        $macro!((RA, RB, RC, RD, RE, RF, RG, RH,), ($($query,)*));
    };
}

macro_rules! count {
    () => {0usize};
    ($head:tt $($tail:tt)*) => {1usize + count!($($tail)*)};
}

#[cfg(test)]
macro_rules! wrap_helper {
    ($var:ident, $var_type:ident, $borrow:expr) => {
        <Ref<$var_type> as WrappableSingle<&$var_type, ()>>::wrap(&mut &$var, &mut $borrow)
    };
    (mut $var:ident, $var_type:ident, $borrow:expr) => {
        <Mut<$var_type> as WrappableSingle<&mut $var_type, ()>>::wrap(&mut &mut $var, &mut $borrow)
    };
}
