use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;

mod utils;


// Edit this function
#[no_mangle]
pub fn derive(spec: Value, model: Value, state: Value, already_updated_state_this_tick: Value) -> String {
    let radius_to_check_in_around_player: f64 = 4.0;
    let point_radius: f64 = 0.5;

    // prep: Compute distances for all points, in respect to their radius
    let player_state = already_updated_state_this_tick.get("player").expect("couldnt read player field");
    let player_coords = player_state.get("position").expect("couldnt read position field");

    let player_coords_x: f64 = player_coords.get("x").expect("").get("value").expect("couldnt read x field")
        .as_f64()
        .expect("Couldn't convert x to f64");

    let player_coords_y: f64 = player_coords.get("y").expect("").get("value").expect("couldnt read y field")
        .as_f64()
        .expect("Couldn't convert y to f64");

    let player_coords_z: f64 = player_coords.get("z").expect("").get("value").expect("couldnt read z field")
        .as_f64()
        .expect("Couldn't convert z to f64");

    struct NearbyPoint {
        coords: Vec<f64>,
        is_gate: bool,
        area_index: usize,
        location_index: usize
    };
    let mut points_nearby: Vec<NearbyPoint> = Vec::new();

    let model_areas = model["areas"].as_array().expect("couldnt convert areas to array");
    for area_index in 0..model_areas.len() {
        let area = &model_areas[area_index];
        let locations_in_current_area = area["locations"].as_array().expect("couldnt convert locations to array");
        for location_index in 0..locations_in_current_area.len() {
            let location = locations_in_current_area.get(location_index).expect("couldnt read actual location");
            let point_cloud = &location["pointCloud"];

            let mut points: HashMap<String, Value> = HashMap::new();
            if let Some(value_array) = point_cloud["points"]["value"].as_array() {
                for pair in value_array.iter() {
                    if let Some(arr) = pair.as_array() {
                        if arr.len() == 2 {
                            let key_str = arr[0].as_str().unwrap();
                            let parsed_key: Value = serde_json::from_str(key_str).unwrap(); // Parse the key JSON string
                            let key_json = serde_json::to_string(&parsed_key).unwrap(); // Normalize key

                            let value: Value = serde_json::from_value(arr[1].clone()).unwrap();
                            points.insert(key_json, value);
                        }
                    }
                }
            }

            let point_keys = points.keys();
            for point_key in point_keys {
                let point = &points[point_key];
                let point_coords = point.get("coords").expect("could read coords field");
                let point_coords_x: f64 = point_coords.get("x").expect("couldnt read x field")
                    .as_f64()
                    .expect("Couldn't convert x to f64");
                let point_coords_y: f64 = point_coords.get("y").expect("couldnt read y field")
                    .as_f64()
                    .expect("Couldn't convert y to f64");

                let point_coords_z: f64 = point_coords.get("z").expect("couldnt read z field")
                    .as_f64()
                    .expect("Couldn't convert z to f64");

                let distance = (
                    (player_coords_x - point_coords_x).powi(2) +
                        (player_coords_y - point_coords_y).powi(2) +
                        (player_coords_z - point_coords_z).powi(2)
                ).sqrt() - radius_to_check_in_around_player - point_radius;

                if distance < 0.0 {
                    let is_gate: bool = location.get("isGateArea").expect("").as_bool().expect("couldnt convert isGateArea to bool");
                    points_nearby.push(NearbyPoint{
                        coords: vec![point_coords_x, point_coords_y, point_coords_z],
                        is_gate,
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
            let area = &model["areas"][nearby_point.area_index];
            let location = &area["locations"][nearby_point.location_index];
            return String::from(location["id"].as_str().expect("Couldn't convert location id"));
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

        let area = &model["areas"][parsed_location_index.area_index];
        let location = &area["locations"][parsed_location_index.location_index];
        return String::from(location.get("id").expect("").as_str().expect(""));
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

#[no_mangle]
pub unsafe fn derive_wrapper(
    ptr_a: *mut u8, len_a: usize,
    ptr_b: *mut u8, len_b: usize,
    ptr_c: *mut u8, len_c: usize,
    ptr_d: *mut u8, len_d: usize,
) -> *mut u8 {
    let data_a = Vec::from_raw_parts(ptr_a, len_a, len_a);
    let input_str_a = String::from_utf8(data_a).unwrap();
    let v1: Value = serde_json::from_str(&*input_str_a).expect("couldn't parse json");

    let data_b = Vec::from_raw_parts(ptr_b, len_b, len_b);
    let input_str_b = String::from_utf8(data_b).unwrap();
    let v2: Value = serde_json::from_str(&*input_str_b).expect("couldn't parse json");

    let data_c = Vec::from_raw_parts(ptr_c, len_c, len_c);
    let input_str_c = String::from_utf8(data_c).unwrap();
    let v3: Value = serde_json::from_str(&*input_str_c).expect("couldn't parse json");

    let data_d = Vec::from_raw_parts(ptr_d, len_d, len_d);
    let input_str_d = String::from_utf8(data_d).unwrap();
    let v4: Value = serde_json::from_str(&*input_str_d).expect("couldn't parse json");


    let derived_result = derive(v1, v2, v3, v4).as_bytes().to_owned();


    let mut raw_bytes = Vec::with_capacity(4 + derived_result.len());
    raw_bytes.extend_from_slice(&derived_result.len().to_le_bytes());
    raw_bytes.extend_from_slice(&derived_result);

    let ptr = raw_bytes.as_mut_ptr();
    std::mem::forget(raw_bytes);
    ptr
}
