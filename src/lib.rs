use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use once_cell::sync::OnceCell;

// global state to optimize some serialization away
static MODEL: OnceCell<Model> = OnceCell::new();

// Edit this function
#[no_mangle]
pub fn derive(model: &Model, already_updated_state_this_tick: &Update) -> String {
    let radius_to_check_in_around_player: f64 = 2.0;
    let point_radius: f64 = 0.5;

    // prep: Compute distances for all points, in respect to their radius
    let player_state = &already_updated_state_this_tick.player;
    let player_coords = &player_state.position;
    let player_coords_x: f64 = player_coords.x.value;
    let player_coords_y: f64 = player_coords.y.value;
    let player_coords_z: f64 = player_coords.z.value;

    struct NearbyPoint {
        coords: Vec<f64>,
        is_gate: bool,
        area_index: usize,
        location_index: usize
    };
    let mut points_nearby: Vec<NearbyPoint> = Vec::new();

    let model_areas = &model.areas;
    for area_index in 0..model_areas.len() {
        let area = &model_areas[area_index];
        let locations_in_current_area = &area.locations;
        for location_index in 0..locations_in_current_area.len() {
            let location = &locations_in_current_area[location_index];
            let point_cloud = &location.pointCloud;

            let mut points: HashMap<String, &ActualPoint> = HashMap::new();
            for pair in point_cloud.points.value.iter() {
                let key_str = &pair.0;
                let parsed_key: Value = serde_json::from_str(key_str).unwrap(); // Parse the key JSON string
                let key_json = serde_json::to_string(&parsed_key).unwrap(); // Normalize key

                points.insert(key_json, &pair.1);
            }

            let point_keys = points.keys();
            for point_key in point_keys {
                let point = points[point_key];
                let point_coords = &point.coords;
                let point_coords_x: f64 = point_coords.x;
                let point_coords_y: f64 = point_coords.y;
                let point_coords_z: f64 = point_coords.z;

                let distance = (
                    (player_coords_x - point_coords_x).powi(2) +
                        (player_coords_y - point_coords_y).powi(2) +
                        (player_coords_z - point_coords_z).powi(2)
                ).sqrt() - radius_to_check_in_around_player - point_radius;

                if distance < 0.0 {
                    points_nearby.push(NearbyPoint{
                        coords: vec![point_coords_x, point_coords_y, point_coords_z],
                        is_gate: location.isGateArea,
                        area_index,
                        location_index
                    });
                }
            }
        }
    }

    // Is there a sphere, that collides and is in a gate area?
    for nearby_point in points_nearby.iter().clone() {
        if nearby_point.is_gate {
            let area = &model.areas[nearby_point.area_index];
            let location = &area.locations[nearby_point.location_index];
            return String::from(&location.id);
        }
    }

    // Else, what area do the majority of the spheres belong to?
    let mut tracker_map: HashMap<String, i32> = HashMap::new();

    for nearby_point in points_nearby {
        let obj_key = format!("{{\"area_index\": {}, \"location_index\": {}}}", nearby_point.area_index, nearby_point.location_index).to_string();
        // Update the value for the given key, defaulting to 0 if it doesn't exist
        let counter = tracker_map.entry(obj_key.clone()).or_insert(0);
        *counter += 1;
    }
    if tracker_map.is_empty() {
        return String::from("error :(");
    }
    let max_entry = tracker_map.iter()
        .max_by_key(|entry| entry.1)
        .map(|entry| entry.clone());

    if let Some(entry) = max_entry {
        #[derive(Deserialize)]
        struct Indexes {
            area_index: usize,
            location_index: usize
        }
        let parsed_location_index: Indexes = serde_json::from_str(entry.0).expect("JSON was not well-formatted");

        let area = &model.areas[parsed_location_index.area_index];
        let location = &area.locations[parsed_location_index.location_index];
        return String::from(&location.id);
    }
    String::from("What the hell:(")
}

/*###################################
        UTILITY FUNCTIONS BELOW
        -----------------------
        FEEL FREE TO TOUCH,
        BUT FROM THAT POINT ON
        YOU ARE ON YOUR OWN!
    ###################################*/

#[no_mangle]
pub fn alloc(len: usize) -> *mut u8 {
    // create a new mutable buffer with capacity `len`
    let mut buf = Vec::with_capacity(len);

    // take a mutable pointer to the buffer
    let ptr = buf.as_mut_ptr();

    // take ownership of the memory block and
    // ensure that its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(buf);

    // return the pointer so the runtime
    // can write data at this offset
    ptr
}

#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let data = Vec::from_raw_parts(ptr, size, size);

    drop(data);
}

// EVEN THOUGH YOU CAN GET VERY SPECIFIC, YOU HAVE TO KEEP THIS PROTOTYPE!!!
#[no_mangle]
pub unsafe fn derive_wrapper(
    _state_pointer: *mut u8, _state_length: usize,
    update_pointer: *mut u8, update_length: usize,
) -> *mut u8 {
    let update_bytes = Vec::from_raw_parts(update_pointer, update_length, update_length);
    let update_string = String::from_utf8(update_bytes).unwrap();
    let parsed_update: Update = serde_json::from_str(&*update_string).expect("couldn't parse update json");


    let derived_result = derive(
        MODEL.get().expect("couldnt unwrap model global"),
        &parsed_update
    ).as_bytes().to_owned();
    // let derived_result = String::from("abcabc").as_bytes().to_owned();

    let mut raw_bytes = Vec::with_capacity(4 + derived_result.len());
    raw_bytes.extend_from_slice(&derived_result.len().to_le_bytes());
    raw_bytes.extend_from_slice(&derived_result);

    let ptr = raw_bytes.as_mut_ptr();
    std::mem::forget(raw_bytes);
    ptr
}


// Dont touch the signature, but you can edit to content to your hearts content
#[no_mangle]
pub unsafe fn derive_setup(
    _spec_pointer: *mut u8, _spec_length: usize,
    model_pointer: *mut u8, model_length: usize,
) {
    let model_bytes = Vec::from_raw_parts(model_pointer, model_length, model_length);
    let model_string = String::from_utf8(model_bytes).unwrap();
    let parsed_model: Model = serde_json::from_str(&*model_string).expect("couldn't parse json");
    let _ = MODEL.set(parsed_model);
}

// Model structs
#[derive(Serialize, Deserialize)]
struct Model {
    areas: Vec<Area>
}

#[derive(Serialize, Deserialize)]
struct Area {
    id: String,
    name: String,
    locations: Vec<Location>
}

#[derive(Serialize, Deserialize)]
struct Location {
    id: String,
    isGateArea: bool,
    pointCloud: PointCloud
}

#[derive(Serialize, Deserialize)]
struct PointCloud {
    points: Points
}

#[derive(Serialize, Deserialize)]
struct Points {
    dataType: String,
    value: Vec<(String, ActualPoint)>
}

#[derive(Serialize, Deserialize)]
struct ActualPoint{
    coords: Coords,
    connections: Vec<Coords>
}

#[derive(Serialize, Deserialize)]
struct Coords {
    x: f64,
    y: f64,
    z: f64,
    angle: f64,
}

//***********
// Update structs
//***********
#[derive(Serialize, Deserialize)]
struct Update {
    player: Player,
}

#[derive(Serialize, Deserialize)]
struct Player {
    position: Position,
    current_location: Option<CurrentLocation>
}

#[derive(Serialize, Deserialize)]
struct Position {
    x: FloatValue,
    y: FloatValue,
    z: FloatValue,
    angle: Option<FloatValue>
}

#[derive(Serialize, Deserialize)]
struct FloatValue {
    value: f64,
}

#[derive(Serialize, Deserialize)]
struct CurrentLocation {
    value: String
}
