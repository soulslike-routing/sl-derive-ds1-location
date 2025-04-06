// Insert your values for testing purposes:

const spec = {};
const model = {/* left out for brevity */};
const stateAsItWasBeforeAtTheStartOfThisTick = {
    player: {
        position: {
            x: {
                value: -52
            },
            y: {
                value: -59,
            },
            z: {
                value: 55
            }
        }
    },
    currentLocation: {
        value: "7e97d14d-770b-44d8-920b-4bea1b8079d7"
    }
};
const alreadyUpdatedStateDuringThisTick = {
    player: {
        position: {
            x: {
                value: -52
            },
            y: {
                value: -59,
            },
            z: {
                value: 55
            }
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
    state,
    update,
    instance
) {
    let stateMemory = jsObjectIntoWasmMemory(state, instance);
    let updateMemory = jsObjectIntoWasmMemory(update, instance);

    // Actually call into wasm derive
    let pointerToResultStruct = instance.exports.derive_wrapper(
        stateMemory.ptr,
        stateMemory.length,
        updateMemory.ptr,
        updateMemory.length
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
    const specPtr = jsObjectIntoWasmMemory(spec, wasm_instance);
    const modelPtr = jsObjectIntoWasmMemory(modelWithMaps, wasm_instance);

    wasm_instance.exports.derive_setup(
        specPtr.ptr, specPtr.length,
        modelPtr.ptr, modelPtr.length
    );

    const deriveString = call_wasm_derive(stateAsItWasBeforeAtTheStartOfThisTick, alreadyUpdatedStateDuringThisTick, wasm_instance);
    console.log(deriveString);
})();