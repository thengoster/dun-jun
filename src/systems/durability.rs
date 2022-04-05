use crate::prelude::*;

#[system(for_each)]
#[write_component(Durability)]
pub fn durability(
    entity: &Entity,
    reduce_durability: &ReduceDurability,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if let Ok(durability) = ecs
        .entry_mut(reduce_durability.entity)
        .unwrap()
        .get_component_mut::<Durability>()
    {
        durability.0 -= 1;
        if durability.0 < 1 {
            commands.remove(reduce_durability.entity);
        }
    }
    commands.remove(*entity);
}
