'use strict';

module.exports = deserialize;

let uint8, uint32, float64, ptrOffset, source;

function deserialize(sourceText, buff) {
    const arrayBuffer = buff.buffer;
    let pos = buff.byteOffset;
    uint8 = pos > 0 ? new Uint8Array(arrayBuffer) : buff;

    uint32 = new Uint32Array(arrayBuffer);
    float64 = new Float64Array(arrayBuffer, 0, arrayBuffer.byteLength >>> 3);

    ptrOffset = uint32[(pos >> 2) + 2];

    source = sourceText;

    const program = deserializeProgram(uint32[pos >> 2]);

    uint8 = uint32 = float64 = undefined;

    return program;
}
