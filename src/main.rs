use bevy::{ecs::observer::TriggerTargets, prelude::*, transform::commands};
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .init_resource::<CursorPos>()
        .add_systems(Startup, startup)
        .add_systems(First, update_cursor_pos)
        .add_systems(Update, get_tile_on_mouse_position)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -1000.0,
            scale: 0.25,
            far: 1000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let texture_handle: Handle<Image> = asset_server.load("prototype-square-atlas.png");
    
    let map_size = TilemapSize { x: 8, y: 8};
    let tile_size = TilemapTileSize { x: 16.0, y: 8.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 8.0 };
    let map_type: TilemapType = TilemapType::Isometric(IsoCoordSystem::Diamond);
    
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
            .spawn(TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            })
            .id();
        tile_storage.set(&tile_pos, tile_entity);
    }
}

commands.entity(tilemap_entity).insert(TilemapBundle {
    grid_size,
    map_type,
    size: map_size,
    storage: tile_storage,
    texture: TilemapTexture::Single(texture_handle),
    tile_size,
    render_settings: TilemapRenderSettings {
            // render_chunk_size: UVec2::new(8, 8),
            y_sort: true,
            ..Default::default()
        },
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
        ..Default::default()
    });
    
    let texture_handle2: Handle<Image> = asset_server.load("prototype-cube-atlas.png");
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tile_size: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    // visible: TileVisible {
                    //     0: false,
                    // },
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle2),
        tile_size,
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::new(3, 1),
            y_sort: true,
            ..Default::default()
        },
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, -1.0) * Transform::from_xyz(0.0, -4.0, 0.0),
        ..Default::default()
    });
    
}

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

fn get_tile_on_mouse_position(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    tile: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform) in tile.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Now we can get the entity at the tile position.
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                commands.entity(tile_entity).insert(TileTextureIndex(1));
                println!(
                    "Tile entity at cursor pos: {:?} , pos {:?}",
                    tile_entity, tile_pos
                );
            }
        }
    }
}
