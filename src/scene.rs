use bevy::{ecs::bundle::DynamicBundle, prelude::*};

use crate::*;

// Constants
pub const FLOOR_Z: f32 = 0.0;
pub const ENTITY_Z_OFFSET: f32 = 10.0;
pub const SCALE: f32 = 4.0;

pub const TILE_WIDTH: f32 = 16.0;
pub const HALF_TILE_WIDTH: f32 = 8.0;
pub const MAP_WIDTH_COORD: f32 = 30.0;
pub const HALF_MAP_WIDTH_COORD: f32 = 15.0;
pub const MAP_WIDTH: f32 = MAP_WIDTH_COORD * TILE_WIDTH;
pub const HALF_MAP_WIDTH: f32 = HALF_MAP_WIDTH_COORD * TILE_WIDTH;

// Plugin
pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(OnEnter(MapLoadState::Done), setup_scene)
            .add_systems(
                OnEnter(SceneState::Transitioning),
                (tear_down_scene, setup_scene).chain(),
            );
    }
}

// Components
#[derive(Debug, Component)]
pub struct NoTearDown;

// States

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SceneState {
    #[default]
    Stable,
    Transitioning,
}

// Systems

fn tear_down_scene(
    mut commands: Commands,
    to_tear_down_query: Query<Entity, (Without<NoTearDown>, Without<Window>)>,
) {
    for entity in to_tear_down_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_scene(
    mut commands: Commands,
    map_server: Res<MapServer>,
    camera_query: Query<&Camera2d>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    let map = &map_server.maps[map_server.map_idx];
    let texture = &map.sprite_sheet.sprite;

    if let Err(err) = camera_query.get_single() {
        if let bevy::ecs::query::QuerySingleError::NoEntities(_) = err {
            commands.spawn((Camera2dBundle::default(), NoTearDown));
        }
    }

    for idx in 0..map.data.len() {
        if map.data[idx] == 0 {
            continue;
        }

        let x = idx % map.width;
        let y = map.width - idx / map.width - 1;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: coord_to_pos(x as f32),
                        y: coord_to_pos(y as f32),
                        z: FLOOR_Z,
                    },
                    scale: Vec3::splat(SCALE),
                    ..default()
                },
                texture: texture.clone(),
                ..default()
            },
            TextureAtlas {
                layout: map.sprite_sheet.texture_atlas_layout.clone(),
                index: (map.data[idx] as usize) - 1,
            },
        ));
    }

    let entity_hydrator = &ComponentHydrators::new()
        .register_hydrator("Mover", hydrate_mover)
        .register_hydrator("Collider", hydrate_collider)
        .register_tag::<Goblinoid>("Goblinoid")
        .register_tag::<Adventurer>("Adventurer")
        .register_tag::<Treasure>("Treasure")
        .register_tag::<Exit>("Exit");

    for obj in map.objects.iter() {
        let sprite_bundle = SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: coord_to_pos(obj.x as f32),
                    y: coord_to_pos(obj.y as f32),
                    z: obj.z + ENTITY_Z_OFFSET,
                },
                scale: Vec3::splat(SCALE),
                ..default()
            },
            texture: texture.clone(),
            ..default()
        };
        let texture_atlas = TextureAtlas {
            layout: obj.sprite_sheet.texture_atlas_layout.clone(),
            index: obj.sprite_idx as usize - 1,
        };

        let mut entity_commands = commands.spawn((sprite_bundle, texture_atlas));

        let components_property = obj.properties.iter().find(|prop| prop.name == "Components");

        if let Some(components_property) = components_property {
            for component_name in String::from(&components_property.value_s).split("|") {
                entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name)
            }
        }
    }
    next_state.set(SceneState::Stable);
}
