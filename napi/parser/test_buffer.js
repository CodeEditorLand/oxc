const oxc = require('./index');
const deserialize = require('./deserialize.js');
const {readFileSync} = require('fs');
const pathJoin = require('path').join;
const assert = require('assert');
const benny = require('benny');
const {filesize} = require('filesize');
const flexbuffers = require('flatbuffers/js/flexbuffers');

function testFlexBuffer(sourceText) {
  const buffer = oxc.parseSyncBuffer(sourceText);
  const ref = flexbuffers.toReference(buffer.buffer);
  assert(ref.isMap());
  assert.equal(ref.get('type').stringValue(), 'Program');
  const body = ref.get('body');
  assert(body.isVector());
}

function testRaw(sourceBuff, allocSize) {
  const buff = oxc.parseSyncRaw(sourceBuff, {}, allocSize);
  const program = deserialize(buff, sourceBuff);
  assert(typeof program === 'object');
  assert.equal(program.type, 'Program');
  assert(Array.isArray(program.body));
}

function testJSON(sourceText) {
  const ret = oxc.parseSync(sourceText);
  const program = JSON.parse(ret.program);
  assert(typeof program === 'object');
  assert.equal(program.type, 'Program');
  assert(Array.isArray(program.body));
}

runAll();

async function runAll() {
  await run('index.js', 64 * 1024); // 64 KiB
  await run('checker.ts', 1024 * 1024); // 1 MiB
  await run('pdf.mjs', 8 * 1024 * 1024); // 8 MiB
  await run('antd.js', 64 * 1024 * 1024); // 64 MiB
}

async function run(filename, allocSize) {
    // Get input code
    const sourceBuff = readFileSync(pathJoin(__dirname, 'fixtures', filename));
    const sourceText = sourceBuff.toString();

    // Run benchmark
    await benny.suite(
        `${filename} (${filesize(sourceBuff.length)})`,

        // Parse
        benny.add('JSON', () => {
            testJSON(sourceText);
        }),

        benny.add('FlexBuffer', () => {
            testFlexBuffer(sourceText);
        }),

        benny.add('Raw', () => {
            testRaw(sourceBuff, allocSize);
        }),

        benny.cycle(),
        benny.complete(),

        // Setting `async` to `true` inserts a pause between cycles.
        // Without this, every run comes straight after the last synchronously
        // which prevents any garbage collection at all. The increased memory usage
        // produces slower performance, so the benchmark is unrealistic.
        // A "cycle" is not a single execution of a bench function, but a batch of
        // a few hundred. This is how benchmark.js works.
        // So it's still using more memory than is realistic. In an async program,
        // memory would likely get freed earlier.
        benny.configure({
            cases: {
                async: true,
            },
        }),

        benny.save({
            file: `bench_${filename}`,
            folder: __dirname,
            details: true,
            format: "chart.html",
        })
    );
}
