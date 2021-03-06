mod components;
mod map_builder;
mod resources;
mod spawner;
mod systems;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    pub use crate::components::*;
    pub use crate::map_builder::*;
    pub use crate::resources::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
}

use prelude::*;

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut ecs, map_builder.player_start);

        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut ecs, &mut rng, 0, &map_builder.monster_spawns);

        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);
        resources.insert(Timer::new());

        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
        }
    }

    fn reset_game_state(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        // spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut self.ecs, &mut rng, 0, &map_builder.monster_spawns);
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(Timer::new());
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(2);
        draw_batch.print_color_centered(2, "Your quest has ended.", ColorPair::new(RED, BLACK));
        <&Player>::query().iter(&self.ecs).for_each(|player| {
            let achieved_score_str = format!("You achieved a score of {}!", player.score);
            draw_batch.print_color_centered(3, achieved_score_str, ColorPair::new(YELLOW, BLACK));
        });

        if let Some(timer) = self.resources.get::<Timer>() {
            let timer_str = format!("You lasted in the dungeon for {}", timer.get_time_string());
            draw_batch.print_color_centered(4, timer_str, ColorPair::new(GOLD, BLACK));
        }

        draw_batch.print_color_centered(
            6,
            "Slain by a monster, your hero's journey has come to a \
            premature end.",
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(
            7,
            "The Amulet of Yala remains unclaimed, and your home town \
            is not saved.",
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(
            8,
            "Don't worry, you can always try again with a new hero.",
            ColorPair::new(YELLOW, BLACK),
        );
        draw_batch.print_color_centered(10, "Press 1 to play again.", ColorPair::new(GREEN, BLACK));

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }

        draw_batch.submit(0).expect("Batch error");
    }

    fn victory(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(2);
        draw_batch.print_color_centered(2, "You have won!", ColorPair::new(GREEN, BLACK));
        <&Player>::query().iter(&self.ecs).for_each(|player| {
            const SCORE_FOR_VICTORY: u32 = 50000;
            let achieved_score_str = format!(
                "You achieved a score of {}!",
                player.score + SCORE_FOR_VICTORY
            );
            draw_batch.print_color_centered(3, achieved_score_str, ColorPair::new(GOLD, BLACK));
        });

        if let Some(timer) = self.resources.get::<Timer>() {
            let timer_str = format!("The Dun-Jun was completed in {}", timer.get_time_string());
            draw_batch.print_color_centered(4, timer_str, ColorPair::new(GOLD, BLACK));
        }

        draw_batch.print_color_centered(
            6,
            "You put on the Amulet of Yala and feel its power course through \
            your veins.",
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(
            7,
            "Your town is saved, and you can return to your normal life.",
            ColorPair::new(WHITE, BLACK),
        );
        draw_batch.print_color_centered(9, "Press 1 to play again.", ColorPair::new(GREEN, BLACK));

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }

        draw_batch.submit(0).expect("Batch error");
    }

    fn advance_level(&mut self) {
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&self.ecs)
            .next()
            .unwrap();

        use std::collections::HashSet;
        let mut entities_to_keep = HashSet::new();
        entities_to_keep.insert(player_entity);
        <(Entity, &Carried)>::query()
            .iter(&self.ecs)
            .filter(|(_e, carry)| carry.0 == player_entity)
            .map(|(e, _carry)| *e)
            .for_each(|e| {
                entities_to_keep.insert(e);
            });

        let mut cb = CommandBuffer::new(&self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        cb.flush(&mut self.ecs);

        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);

        let mut map_level = 0;
        <(&mut Player, &mut Point)>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|(player, pos)| {
                player.map_level += 1;
                map_level = player.map_level;
                player.score += 10000;
                pos.x = map_builder.player_start.x;
                pos.y = map_builder.player_start.y;
            });

        if map_level == 2 {
            spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start)
        } else {
            let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
            map_builder.map.tiles[exit_idx] = TileType::Exit;
        }

        spawn_level(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            &map_builder.monster_spawns,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        // self.resources.insert(Timer::new()); TODO: Timer is dungeon timer, not per level...maybe have two timers in future?
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();

        self.resources.insert(ctx.key);
        self.resources.insert(ctx.frame_time_ms);
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));

        let current_state = *self.resources.get::<TurnState>().unwrap();
        match current_state {
            TurnState::AwaitingInput => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => self
                .player_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::MonsterTurn => self
                .monster_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::GameOver => self.game_over(ctx),
            TurnState::Victory => self.victory(ctx),
            TurnState::NextLevel => self.advance_level(),
        }
        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Dun-Jun")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "terminal8x8.png")
        .build()?;
    main_loop(context, State::new())
}
