'use strict';

const customDeserializers = {
    Language: deserializeLanguage,
    RegExpFlags: deserializeRegExpFlags,
    ReferenceFlag: deserializeReferenceFlag,
};

module.exports = {customDeserializers, generateCustomDeserializer};

function generateCustomDeserializer(type) {
    return customDeserializers[type.name].toString();
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
