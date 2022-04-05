use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[write_component(FieldOfView)]
#[read_component(Carried)]
#[read_component(ProvidesDigging)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let mut move_entity = false;

    if map.can_enter_tile(want_move.destination) {
        move_entity = true;
    } else if map.in_bounds(want_move.destination) {
        if let Some(shovel) = <(Entity, &Carried)>::query()
            .filter(component::<ProvidesDigging>())
            .iter(ecs)
            .filter(|(_, carried)| carried.0 == want_move.entity)
            .find_map(|(entity, _)| Some(*entity))
        {
            let idx = map.point2d_to_index(want_move.destination);
            map.tiles[idx] = TileType::Floor;
            move_entity = true;

            commands.push(((), ReduceDurability { entity: shovel }));
        }
    }

    if move_entity {
        commands.add_component(want_move.entity, want_move.destination);

        if let Ok(entry) = ecs.entry_ref(want_move.entity) {
            if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok() {
                    camera.on_player_move(want_move.destination);
                    fov.visible_tiles.iter().for_each(|pos| {
                        map.revealed_tiles[map_idx(pos.x, pos.y)] = true;
                    });
                }
            }
        }
    }
    // removes message entity
    commands.remove(*entity);
}
