'use strict';

const customDeserializers = {
    Atom: deserializeAtom,
    RefStr: deserializeRefStr,
    Language: deserializeLanguage,
    RegExpFlags: deserializeRegExpFlags,
    ReferenceFlag: deserializeReferenceFlag,
};

module.exports = {customDeserializers, generateCustomDeserializer};

function generateCustomDeserializer(type) {
    return customDeserializers[type.name].toString();
}

function deserializeAtom(pos) {
    if (uint8[pos] !== 0) throw new Error(`Unexpected discriminant ${uint8[pos]} for Atom`);

    const pos32 = pos >> 2,
        len = uint32[pos32 + 4];
    if (len === 0) return '';
    
    const strLow = uint32[pos32 + 2],
        strHigh = uint32[pos32 + 3];
    let strBuff;
    if (strHigh === ptrHigh && strLow >= ptrOffset && strLow < endLow) {
        // String is in buffer
        const offset = strLow - ptrOffset;
        strBuff = uint8.subarray(offset, offset + len);
    } else {
        // String is in source
        let offset = strLow - sourceLow;
        if (strHigh > sourceHigh) offset += 4294967296; // 1 << 32
        strBuff = source.subarray(offset, offset + len);
    }

    return textDecoder.decode(strBuff);
}

function deserializeRefStr(pos) {
    const pos32 = pos >> 2,
        len = uint32[pos32 + 2];
    if (len === 0) return '';
    
    const strLow = uint32[pos32],
        strHigh = uint32[pos32 + 1];
    let offset = strLow - sourceLow;
    if (strHigh > sourceHigh) offset += 4294967296; // 1 << 32

    const strBuff = source.subarray(offset, offset + len);
    return textDecoder.decode(strBuff);
}

// TODO: Make this be generated automatically
function deserializeLanguage(pos) {
    switch (uint8[pos]) {
        case 0: return 'javaScript';
        case 1: return {typeScript: {isDefinitionFile: uint8[pos + 1] === 1}};
        default: throw new Error(`Unexpected discriminant ${uint8[pos]} for Language`);
    }
}

function deserializeRegExpFlags(pos) {
    const bits = uint8[pos];
    let text = '';
    if (bits & 1) text += 'g';
    if (bits & 2) text += 'i';
    if (bits & 4) text += 'm';
    if (bits & 8) text += 's';
    if (bits & 16) text += 'u';
    if (bits & 32) text += 'y';
    if (bits & 64) text += 'd';
    if (bits & 128) text += 'v';
    return text;
}

function deserializeReferenceFlag(pos) {
    const bits = uint8[pos],
        parts = [];
    if (bits & 1) parts.push('Read');
    if (bits & 2) parts.push('Write');
    if (bits & 4) parts.push('Type');
    return parts.join(' | ');
}
