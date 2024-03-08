'use strict';

const {readFileSync, writeFileSync, mkdirSync, rmSync} = require('fs'),
  pathJoin = require('path').join,
  {spawnSync} = require('child_process'),
  {expect} = require('expect'),
  oxc = require('./index.js'),
  deserialize = require('./deserialize.js');

console.log(`Testing on ${process.platform}-${process.arch}`)

test('index.js', 64 * 1024); // 64 KiB
test('checker.ts', 1024 * 1024); // 1 MiB
test('pdf.mjs', 8 * 1024 * 1024); // 8 MiB
test('antd.js', 64 * 1024 * 1024); // 64 MiB

function test(filename, allocSize) {
  console.log('Testing:', filename);

  const sourceBuff = readFileSync(pathJoin(__dirname, 'fixtures', filename)),
    sourceText = sourceBuff.toString();

  const astViaJson = JSON.parse(oxc.parseSync(sourceText).program);
  // console.log(astViaJson);

  const buff = oxc.parseSyncRaw(sourceBuff, {}, allocSize);
  const astRaw = deserialize(buff, sourceBuff);
  // console.log(astRaw);

  if (JSON.stringify(astRaw) === JSON.stringify(astViaJson)) {
    console.log('> Pass');
  } else {
    console.log('> Fail');

    const diffPath = pathJoin(__dirname, 'diffTemp');
    mkdirSync(diffPath, {recursive: true});
    const pathJson = pathJoin(diffPath, `${filename}.json.json`),
      pathRaw = pathJoin(diffPath, `${filename}.raw.json`);
    writeFileSync(pathJson, JSON.stringify(astViaJson, null, 2));
    writeFileSync(pathRaw, JSON.stringify(astRaw, null, 2));
    const diff = spawnSync('diff', [pathJson, pathRaw]).stdout;
    writeFileSync(pathJoin(__dirname, `${filename}.diff`), diff);
    rmSync(diffPath, {recursive: true});
  }

  // assertEqual(astRaw, astViaJson);
}

function assertEqual(val1, val2) {
  try {
    expect(val1).toEqual(val2);
  } catch (err) {
    delete err.matcherResult;
    throw err;
  }
}
