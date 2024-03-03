'use strict';

const fs = require('fs'),
    pathJoin = require('path').join,
    assert = require('assert'),
    {format} = require('@prettier/sync'),
    oxc = require('../index.js');

const generatePrimitiveDeserializer = require('./primitives.js'),
    {
        generateStructDeserializer,
        generateStructFieldCode,
        generateEnumDeserializer,
        generateBoxDeserializer,
        generateVecDeserializer,
        generateOptionDeserializer
    } = require('./structs.js'),
    {customDeserializers, generateCustomDeserializer} = require('./custom.js');

console.log(`Generating deserializer on ${process.platform}-${process.arch}`)

// Get schema
let types = JSON.parse(oxc.getSchema());

// Conform type names and flatten cells
let typesByName = Object.create(null);
for (const [index, type] of types.entries()) {
    // Flatten cells + transparent structs
    if (type.kind === 'cell') {
        types[index] = types[type.valueTypeId];
        continue;
    }
    if (type.kind === 'struct' && type.transparent && type.fields[0].offset === 0) {
        types[index] = types[type.fields[0].typeId];
        continue;
    }

    // Conform type name
    type.name = type.name
        .replace(/<(.)/g, (_, c) => c.toUpperCase())
        .replace(/[>, ]/g, '')
        .replace(/^(.)/, (_, c) => c.toUpperCase());
    assert(!typesByName[type.name], `Repeated type name ${type.name}`);
    typesByName[type.name] = type;

    type.dependencies = new Set();
    type.niche = null;
    type.deserializerName = `deserialize${type.name}`;
    type.isOutput = false;
}

// Link up types.
// Delete skipped fields from structs.
const structs = [];
for (const type of Object.values(typesByName)) {
    if (type.kind === 'struct') {
        // Remove skipped fields, and get types for fields
        type.fields = type.fields.flatMap((field) => {
            const {serName, typeId, skip, name: _name, ...fieldProps} = field;
            if (skip) return [];
            return {name: serName, type: types[typeId], ...fieldProps};
        });
        structs.push(type);
    } else if (type.kind === 'enum') {
        // Get types for variants
        let minDiscriminant = Infinity, maxDiscriminant = 0, numTypedVariants = 0;
        type.variants = type.variants.map((variant) => {
            const {valueTypeId, discriminant, ...variantProps} = variant;
            if (discriminant < minDiscriminant) minDiscriminant = discriminant;
            if (discriminant > maxDiscriminant) maxDiscriminant = discriminant;

            let variantType = null;
            if (valueTypeId !== null) {
                variantType = types[valueTypeId];
                type.dependencies.add(variantType);
                numTypedVariants++;
            }
            return {discriminant, type: variantType, ...variantProps};
        });

        // Calculate niche
        type.niche = {
            offset: 0,
            size: 1,
            value: minDiscriminant == 0 ? maxDiscriminant + 1 : 0
        };

        // Check either all variants are typed, or none are
        if (numTypedVariants === 0) {
            type.isTyped = false;
        } else {
            assert(numTypedVariants === type.variants.length);
            type.isTyped = true;
        }
    } else if (type.kind === 'vec' || type.kind === 'box' || type.kind === 'option') {
        const childType = types[type.valueTypeId];
        delete type.valueTypeId;
        type.type = childType;
        type.dependencies.add(childType);

        if (type.kind !== 'option') {
            type.niche = {offset: 0, size: 8, value: 0};
        }
    } else if (type.kind === 'primitive') {
        if (type.name === 'Bool') {
            type.niche = {offset: 0, size: 1, value: 2};
        } else if (type.name.startsWith('NonZero')) {
            type.niche = {offset: 0, size: type.size, value: 2};
        }
    } else {
        assert(type.kind === 'strSlice', `Unexpected type kind '${type.kind}'`);
        type.niche = {offset: 0, size: 8, value: 0};
    }
}

// Flatten struct fields tagged with `serde(flatten)` + get dependencies for structs
for (const type of structs) {
    if (type.kind !== 'struct') continue;

    const {fields} = type;
    for (let i = 0; i < fields.length; i++) {
        const field = fields[i];
        if (field.flatten) {
            fields.splice(
                i, 1,
                ...field.type.fields.map(child => ({...child, offset: field.offset + child.offset}))
            );
            // Go over these fields again, in case they're recursively flattened
            i--;
            continue;
        }

        delete field.flatten;
        type.dependencies.add(field.type);
    }
}

// Define niches for various types
function setNicheForStructFromField(typeName, fieldName) {
    const type = typesByName[typeName],
        field = type.fields.find(field => field.name === fieldName),
        child = field.type;
    type.niche = {...child.niche, offset: child.niche.offset + field.offset};
}

typesByName.Atom.niche = {offset: 0, size: 1, value: 255};

setNicheForStructFromField('Hashbang', 'value');
setNicheForStructFromField('LabelIdentifier', 'name');
setNicheForStructFromField('BindingPattern', 'optional');
setNicheForStructFromField('IdentifierName', 'name');
setNicheForStructFromField('TSThisParameter', 'this');
setNicheForStructFromField('BindingIdentifier', 'name');
setNicheForStructFromField('WithClause', 'attributesKeyword');
setNicheForStructFromField('StringLiteral', 'value');
setNicheForStructFromField('TSImportAttributes', 'elements');

// Set custom types
for (const typeName in customDeserializers) {
    typesByName[typeName].kind = 'custom';
}

// Set types containing `Atom` to use span `start` and `end` to extract string
function adaptTypeForSpan(type) {
    const startField = type.fields.find(field => field.name === 'start'),
        endField = type.fields.find(field => field.name === 'end');
    type.preamble = `const start = ${generateStructFieldCode(startField)},
        end = ${generateStructFieldCode(endField)};`;
    startField.code = 'start';
    endField.code = 'end';
    type.dependencies.delete(typesByName.Atom);
}

function adaptTypeContainingAtom(typeName, sliceStart, sliceEnd) {
    const type = typesByName[typeName];
    adaptTypeForSpan(type);
    const atomField = type.fields.find(field => field.type === typesByName.Atom);
    atomField.code = `source.slice(${sliceStart}, ${sliceEnd})`;
}

function addVarToPreamble(type, code) {
    type.preamble = `${type.preamble.slice(0, -1)},${code};`;
}

adaptTypeContainingAtom('StringLiteral', 'start + 1', 'end - 1');
adaptTypeContainingAtom('Directive', 'start + 1', 'end - 2'); // `-2` to exclude semi-colon
adaptTypeContainingAtom('Hashbang', 'start + 2', 'end');
adaptTypeContainingAtom('LabelIdentifier', 'start', 'end');
adaptTypeContainingAtom('BigintLiteral', 'start', 'end');
adaptTypeContainingAtom('IdentifierReference', 'start', 'end');
adaptTypeContainingAtom('IdentifierName', 'start', 'end');
adaptTypeContainingAtom('BindingIdentifier', 'start', 'end');
adaptTypeContainingAtom('PrivateIdentifier', 'start + 1', 'end');
adaptTypeContainingAtom('TSIndexSignatureName', 'start', 'end');
adaptTypeContainingAtom('JSXIdentifier', 'start', 'end');
adaptTypeContainingAtom('JSXText', 'start', 'end');

// TemplateElement
{
    const type = typesByName.TemplateElement,
        valueField = type.fields.find(field => field.name === 'value');
    adaptTypeForSpan(type);
    addVarToPreamble(type, 'text = source.slice(start, end)');
    valueField.code = `{raw: text, cooked: text}`;
    type.dependencies.delete(valueField.type);
}

// RegExpLiteral
{
    const type = typesByName.RegExpLiteral,
        regexField = type.fields.find(field => field.name === 'regex'),
        flagsField = regexField.type.fields.find(field => field.name === 'flags');
    adaptTypeForSpan(type);
    addVarToPreamble(type, `flags = ${generateStructFieldCode(flagsField)}`);

    regexField.code = `{pattern: source.slice(start + 1, end - flags.length - 1), flags}`
    type.dependencies.delete(regexField.type);
    type.dependencies.add(flagsField.type);
}

// Generate deserializer
let code = '// Code generated by `generate/index.js`. Do not edit.\n\n'
    + fs.readFileSync(pathJoin(__dirname, 'base.js')) + '\n';

const generators = {
    primitive: generatePrimitiveDeserializer,
    struct: generateStructDeserializer,
    enum: generateEnumDeserializer,
    box: generateBoxDeserializer,
    vec: generateVecDeserializer,
    option: generateOptionDeserializer,
    custom: generateCustomDeserializer,
};

function generateDeserializer(type, parent) {
    if (type.isOutput) return;
    type.isOutput = true;

    code += generators[type.kind](type) + '\n\n';

    for (const childType of type.dependencies) {
        generateDeserializer(childType, type);
    }
}
generateDeserializer(typesByName.Program, null);

code = format(code, {filepath: '.js', tabWidth: 4, singleQuote: true, printWidth: 100});

fs.writeFileSync(pathJoin(__dirname, '../deserialize.js'), code);
