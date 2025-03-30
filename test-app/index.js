// Insert your values for testing purposes:

const spec = {};
const model = {};
const stateAsItWasBeforeAtTheStartOfThisTick = {};
const alreadyUpdatedStateDuringThisTick = {
    player: {
        position: {
            x: -52,
            y: -59,
            z: 55
        }
    }
}

/*###################################
        UTILITY FUNCTIONS BELOW
        -----------------------
        FEEL FREE TO TOUCH,
        BUT FROM THAT POINT ON
        YOU ARE ON YOUR OWN!
    ###################################*/

const base64_promise = fetch("assets/wasm_base64.txt");
const modelWithMaps = JSON.parse(JSON.stringify(model), reviver);

function replacer(key, value) {
    if(value instanceof Map) {
        return {
            dataType: 'Map',
            value: Array.from(value.entries()),
        };
    } else {
        return value;
    }
}

function reviver(key, value) {
    if(typeof value === 'object' && value !== null) {
        if (value.dataType === 'Map') {
            return new Map(value.value);
        }
    }
    return value;
}

function readStringWith4PrependedLengthBytes(ptr, instance) {
    var memory = new Uint8Array(instance.exports.memory.buffer);

    const view = new DataView(memory.buffer, ptr, 4);
    const length = view.getUint32(0, true); // true -> little-endian
    console.log(length);

    var decoder = new TextDecoder("utf-8");

    var str = decoder.decode(memory.subarray(ptr+4, ptr+4+length));
    return { string: str, bytes: length };
}

function jsObjectIntoWasmMemory(jsObject, instance) {
    const jsObjectAsString = JSON.stringify(jsObject, replacer);
    const jsObjectAsBytes = new TextEncoder("utf-8").encode(jsObjectAsString);
    let ptrToWasmMemory = instance.exports.alloc(jsObjectAsBytes.length);
    let memoryBuffer = new Uint8Array(instance.exports.memory.buffer, ptrToWasmMemory, jsObjectAsBytes.length);
    memoryBuffer.set(new Uint8Array(jsObjectAsBytes));
    return {ptr: ptrToWasmMemory, length: jsObjectAsBytes.length};
}

function call_wasm_derive(
    input1,
    input2,
    input3,
    input4,
    instance
) {
    let input1Struct = jsObjectIntoWasmMemory(input1, instance);
    let input2Struct = jsObjectIntoWasmMemory(input2, instance);
    let input3Struct = jsObjectIntoWasmMemory(input3, instance);
    let input4Struct = jsObjectIntoWasmMemory(input4, instance);

    // Actually call into wasm derive
    let pointerToResultStruct = instance.exports.derive_wrapper(
        input1Struct.ptr,
        input1Struct.length,
        input2Struct.ptr,
        input2Struct.length,
        input3Struct.ptr,
        input3Struct.length,
        input4Struct.ptr,
        input4Struct.length
    );

    const resultStruct = readStringWith4PrependedLengthBytes(pointerToResultStruct, instance);
    instance.exports.dealloc(pointerToResultStruct, 4 + resultStruct.bytes);
    return resultStruct.string;
}

async function wasm_instance_from_b64_string(b64wasm) {
    const binaryString = atob(b64wasm);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }

    const mod = new WebAssembly.Module(bytes);
    return await WebAssembly.instantiate(mod, {});
}

(async () => {
    const base64_string = await (await base64_promise).text();
    let wasm_instance = await wasm_instance_from_b64_string(base64_string);
    const deriveString = call_wasm_derive(spec, modelWithMaps, stateAsItWasBeforeAtTheStartOfThisTick, alreadyUpdatedStateDuringThisTick, wasm_instance);
    console.log(deriveString);
})();