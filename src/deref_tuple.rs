use crate::{Ref, RefMut};

pub trait DerefTuple<'a> {
    type Output;

    unsafe fn deref(&mut self) -> Self::Output;
}

impl<'a, R0> DerefTuple<'a> for Ref<R0>
where
    R0: 'a,
{
    type Output = &'a R0;

    unsafe fn deref(&mut self) -> Self::Output {
        std::mem::transmute(Ref::<R0>::deref(self))
    }
}

impl<'a, R0> DerefTuple<'a> for RefMut<R0>
where
    R0: 'a,
{
    type Output = &'a mut R0;

    unsafe fn deref(&mut self) -> Self::Output {
        std::mem::transmute(RefMut::<R0>::deref(self))
    }
}

impl<'a> DerefTuple<'a> for () {
    type Output = ();

    unsafe fn deref(&mut self) -> Self::Output {}
}

impl<'a, D0> DerefTuple<'a> for (D0,)
where
    D0: DerefTuple<'a>,
{
    type Output = (D0::Output,);

    unsafe fn deref(&mut self) -> Self::Output {
        (self.0.deref(),)
    }
}

macro_rules! impl_deref_tuple {
    ($($letter:ident),*) => {
        impl<'a, $($letter,)*> DerefTuple<'a> for ($($letter,)*)
        where
            $($letter: DerefTuple<'a>,)*
        {
            type Output = ($($letter::Output,)*);

            #[allow(non_snake_case)]
            unsafe fn deref(&mut self) -> Self::Output {
                let ($($letter,)*) = self;
                ($($letter.deref(),)*)
            }
        }
    }
}

impl_for_tuples!(impl_deref_tuple);
