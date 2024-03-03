'use strict';

const {writeFileSync} = require('fs'),
  pathJoin = require('path').join,
  oxc = require('./index.js');

const schemaArr = JSON.parse(oxc.getSchema());
const schema = Object.fromEntries(schemaArr.map(entry => [entry.name, entry]));
console.log(schema);

writeFileSync(pathJoin(__dirname, 'schema.json'), JSON.stringify(schema, null, 2));
