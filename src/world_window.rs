use crate::World;

pub struct WorldWindow<'a> {
    pub(crate) world: &'a World,
}

impl<'a> WorldWindow<'a> {
    pub(crate) fn new(world: &'a World) -> Self {
        Self { world }
    }
}
