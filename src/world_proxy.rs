use crate::World;

#[derive(Clone, Copy)]
pub struct WorldProxy<'a> {
    pub(crate) world: &'a World,
}

impl<'a> WorldProxy<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }
}
