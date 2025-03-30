use serde_json::Value;

mod utils;


// Edit this function
#[no_mangle]
pub fn derive(spec: Value, model: Value, state: Value, already_updated_state_this_tick: Value) -> String {

    String::from("YOUR CODE HERE")

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
