// Generate a minimal valid binary glTF (GLB) containing one triangle.
// Usage: node make_sample_glb.js <output.glb>
const fs = require('fs');

const positions = new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]);
const bin = Buffer.from(positions.buffer, positions.byteOffset, positions.byteLength);

const json = JSON.stringify({
  asset: { version: '2.0', generator: 'kasuga-converter-e2e' },
  scene: 0,
  scenes: [{ nodes: [0] }],
  nodes: [{ mesh: 0 }],
  meshes: [{ primitives: [{ attributes: { POSITION: 0 }, mode: 4 }] }],
  accessors: [
    {
      bufferView: 0,
      componentType: 5126,
      count: 3,
      type: 'VEC3',
      min: [0, 0, 0],
      max: [1, 1, 0],
    },
  ],
  bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: bin.length }],
  buffers: [{ byteLength: bin.length }],
});

let jsonBuf = Buffer.from(json, 'utf8');
const jsonPad = (4 - (jsonBuf.length % 4)) % 4;
if (jsonPad) jsonBuf = Buffer.concat([jsonBuf, Buffer.alloc(jsonPad, 0x20)]);
const binPad = (4 - (bin.length % 4)) % 4;
const binBuf = binPad ? Buffer.concat([bin, Buffer.alloc(binPad, 0)]) : bin;

const total = 12 + 8 + jsonBuf.length + 8 + binBuf.length;
const header = Buffer.alloc(12);
header.write('glTF', 0, 'ascii');
header.writeUInt32LE(2, 4);
header.writeUInt32LE(total, 8);

const jsonHeader = Buffer.alloc(8);
jsonHeader.writeUInt32LE(jsonBuf.length, 0);
jsonHeader.write('JSON', 4, 'ascii');

const binHeader = Buffer.alloc(8);
binHeader.writeUInt32LE(binBuf.length, 0);
binHeader.write('BIN\0', 4, 'ascii');

const out = process.argv[2] || 'sample.glb';
fs.writeFileSync(out, Buffer.concat([header, jsonHeader, jsonBuf, binHeader, binBuf]));
console.log(`wrote ${out} (${total} bytes)`);
