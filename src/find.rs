use fastanvil::Region;
use fastnbt::{from_bytes, Value};
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct DroppedItem {
    pub item_type: String,
    pub position: (f64, f64, f64),
    pub count: i32,
    pub entity_file: String,
    pub age: Option<i16>,
}

#[derive(Deserialize, Debug)]
struct EntityData {
    #[serde(rename = "Entities")]
    entities: Vec<Entity>,
}

#[derive(Deserialize, Debug)]
struct Entity {
    id: String,
    #[serde(rename = "Pos")]
    pos: Vec<f64>,
    #[serde(rename = "Item")]
    item: Option<ItemData>,
    #[serde(rename = "Age")]
    age: Option<i16>,
}

#[derive(Deserialize, Debug)]
struct ItemData {
    id: String,
    count: i8,
}

pub fn find_dropped_items_in_entity_file(entity_path: &Path) -> Vec<DroppedItem> {
    let mut dropped_items = Vec::new();

    let file = File::open(entity_path).unwrap_or_else(|e| {
        panic!("Cannot open entity file {}: {}", entity_path.display(), e);
    });

    let mut region = match Region::from_stream(file) {
        Ok(region) => region,
        Err(e) => {
            eprintln!("Skipping {}: {}", entity_path.display(), e);
            return Vec::new();
        }
    };

    for chunk in region.iter().flatten() {
        match from_bytes::<EntityData>(&chunk.data) {
            Ok(entity_data) => {
                for entity in entity_data.entities {
                    if entity.id == "minecraft:item" {
                        if let Some(item) = entity.item {
                            if entity.pos.len() >= 3 {
                                dropped_items.push(DroppedItem {
                                    item_type: item.id,
                                    position: (entity.pos[0], entity.pos[1], entity.pos[2]),
                                    count: item.count as i32,
                                    entity_file: entity_path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string(),
                                    age: entity.age,
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if let Ok(raw_data) = from_bytes::<Value>(&chunk.data) {
                    let debug_filename = format!(
                        "entity_structure_debug_{}.json",
                        chrono::Local::now().format("%Y%m%d_%H%M%S")
                    );
                    std::fs::write(&debug_filename, format!("{:#?}", raw_data)).unwrap_or_else(
                        |e| {
                            panic!("Failed to save debug file: {}", e);
                        },
                    );
                    panic!(
                        "Failed to parse entity data: {}. Structure saved to file {}",
                        e, debug_filename
                    );
                } else {
                    panic!("Complete failure in parsing entity data: {}", e);
                }
            }
        }
    }

    dropped_items
}
