// Generate a textured binary FBX fixture set into a directory:
//   sample_tex.fbx  (binary FBX 7.4, one textured quad)
//   texture.png     (4x4 checkerboard, referenced from the FBX)
// Usage: node make_sample_fbx_tex.js <output_dir>
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

const outDir = process.argv[2];
if (!outDir) { console.error('usage: node make_sample_fbx_tex.js <dir>'); process.exit(1); }
fs.mkdirSync(outDir, { recursive: true });

// ---- texture.png (4x4 checkerboard, truecolor) ----
function crc32(buf) {
  let c = ~0;
  for (const x of buf) { c ^= x; for (let i = 0; i < 8; i++) c = c & 1 ? (c >>> 1) ^ 0xEDB88320 : c >>> 1; }
  return ~c >>> 0;
}
function pngChunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type), data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}
{
  const w = 4, h = 4;
  const raw = Buffer.alloc(h * (w * 3 + 1));
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const o = y * (w * 3 + 1) + 1 + x * 3;
    const on = (x + y) % 2 === 0;
    raw[o] = on ? 255 : 30; raw[o + 1] = 0; raw[o + 2] = on ? 0 : 255;
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4); ihdr[8] = 8; ihdr[9] = 2;
  fs.writeFileSync(path.join(outDir, 'texture.png'), Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk('IHDR', ihdr),
    pngChunk('IDAT', zlib.deflateSync(raw)),
    pngChunk('IEND', Buffer.alloc(0)),
  ]));
}

// ---- sample_tex.fbx (binary FBX 7.4) ----
const out = [];
let len = 0;
const push = (b) => { out.push(b); len += b.length; };
const u8 = (v) => push(Buffer.from([v & 0xff]));
const u32 = (v) => { const b = Buffer.alloc(4); b.writeUInt32LE(v >>> 0); push(b); };
const i32 = (v) => { const b = Buffer.alloc(4); b.writeInt32LE(v); push(b); };
const i64 = (v) => { const b = Buffer.alloc(8); b.writeBigInt64LE(BigInt(v)); push(b); };
const f64 = (v) => { const b = Buffer.alloc(8); b.writeDoubleLE(v); push(b); };
const str = (s) => { const b = Buffer.from(s, 'latin1'); u32(b.length); push(b); };
const pS = (s) => { u8(0x53); str(s); };
const pI = (v) => { u8(0x49); i32(v); };
const pL = (v) => { u8(0x4c); i64(v); };
const pD = (v) => { u8(0x44); f64(v); };
const pArrD = (a) => { u8(0x64); const b = Buffer.alloc(a.length * 8); a.forEach((v, i) => b.writeDoubleLE(v, i * 8)); u32(a.length); u32(0); u32(b.length); push(b); };
const pArrI = (a) => { u8(0x69); const b = Buffer.alloc(a.length * 4); a.forEach((v, i) => b.writeInt32LE(v, i * 4)); u32(a.length); u32(0); u32(b.length); push(b); };
function propLen(p) {
  const t = p[0], b = p[1];
  if (t === 'S' || t === 'R') return 1 + 4 + Buffer.byteLength(b, 'latin1');
  if (t === 'I' || t === 'F') return 5;
  if (t === 'C') return 2;
  if (t === 'D' || t === 'L') return 9;
  if (t === 'd' || t === 'l') return 1 + 12 + b.length * 8;
  if (t === 'i') return 1 + 12 + b.length * 4;
  throw new Error('prop ' + t);
}
function writeProp(p) {
  const t = p[0];
  if (t === 'S') pS(p[1]); else if (t === 'I') pI(p[1]); else if (t === 'L') pL(p[1]);
  else if (t === 'D') pD(p[1]); else if (t === 'C') { u8(0x43); u8(p[1] ? 1 : 0); }
  else if (t === 'd') pArrD(p[1]); else if (t === 'i') pArrI(p[1]);
  else if (t === 'R') { u8(0x52); str(p[1]); }
  else throw new Error('prop ' + t);
}
function nodeSize(n) {
  let s = 13 + Buffer.byteLength(n.name, 'latin1');
  for (const p of n.props) s += propLen(p);
  for (const c of n.children) s += nodeSize(c);
  if (n.children.length) s += 13;
  return s;
}
function writeNode(n, base) {
  const size = nodeSize(n);
  u32(base + size); u32(n.props.length);
  let plen = 0; for (const p of n.props) plen += propLen(p);
  u32(plen); u8(Buffer.byteLength(n.name, 'latin1')); push(Buffer.from(n.name, 'latin1'));
  for (const p of n.props) writeProp(p);
  let off = base + 13 + Buffer.byteLength(n.name, 'latin1') + plen;
  for (const c of n.children) { writeNode(c, off); off += nodeSize(c); }
  if (n.children.length) push(Buffer.alloc(13));
}
const N = (name, props, children) => ({ name, props, children: children || [] });
const SEP = '\x00\x01';
const tex = 'texture.png';

const tree = [
  N('FBXHeaderExtension', [], [
    N('FBXHeaderVersion', [['I', 1003]]),
    N('FBXVersion', [['I', 7400]]),
    N('EncryptionType', [['I', 0]]),
    N('CreationTimeStamp', [], [
      N('Version', [['I', 1000]]), N('Year', [['I', 2026]]), N('Month', [['I', 11]]),
      N('Day', [['I', 16]]), N('Hour', [['I', 12]]), N('Minute', [['I', 0]]),
      N('Second', [['I', 0]]), N('Millisecond', [['I', 0]]),
    ]),
    N('Creator', [['S', 'kasuga_converter fixture']]),
  ]),
  N('FileId', [['R', '0123456789abcdef']]),
  N('Creator', [['S', 'kasuga_converter fixture']]),
  N('GlobalSettings', [], [
    N('Version', [['I', 1000]]),
    N('Properties70', [], [
      N('P', [['S', 'UpAxis'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 1]]),
      N('P', [['S', 'UpAxisSign'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 1]]),
      N('P', [['S', 'FrontAxis'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 2]]),
      N('P', [['S', 'FrontAxisSign'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 1]]),
      N('P', [['S', 'CoordAxis'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 0]]),
      N('P', [['S', 'CoordAxisSign'], ['S', 'int'], ['S', 'Integer'], ['S', ''], ['I', 1]]),
      N('P', [['S', 'UnitScaleFactor'], ['S', 'double'], ['S', 'Number'], ['S', ''], ['D', 100]]),
    ]),
  ]),
  N('Documents', [], [
    N('Count', [['I', 1]]),
    N('Document', [['L', 140000000000001], ['S', 'Scene' + SEP + 'Document'], ['S', 'Scene']], [
      N('Properties70', [], [N('P', [['S', 'SourceObject'], ['S', 'object'], ['S', ''], ['S', '']])]),
      N('RootNode', [['L', 0]]),
    ]),
  ]),
  N('References', [], []),
  N('Definitions', [], [
    N('Version', [['I', 100]]),
    N('Count', [['I', 5]]),
    N('ObjectType', [['S', 'Geometry']], [N('Count', [['I', 1]])]),
    N('ObjectType', [['S', 'Model']], [N('Count', [['I', 1]])]),
    N('ObjectType', [['S', 'Material']], [N('Count', [['I', 1]])]),
    N('ObjectType', [['S', 'Texture']], [N('Count', [['I', 1]])]),
    N('ObjectType', [['S', 'Video']], [N('Count', [['I', 1]])]),
  ]),
  N('Objects', [], [
    N('Geometry', [['L', 1234567], ['S', SEP + 'Geometry'], ['S', 'Mesh']], [
      N('Vertices', [['d', [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0]]]),
      N('PolygonVertexIndex', [['i', [0, 1, 2, -4]]]),
      N('GeometryVersion', [['I', 124]]),
      N('LayerElementNormal', [['I', 0]], [
        N('Version', [['I', 102]]), N('Name', [['S', '']]),
        N('MappingInformationType', [['S', 'ByPolygonVertex']]),
        N('ReferenceInformationType', [['S', 'Direct']]),
        N('Normals', [['d', [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1]]]),
      ]),
      N('LayerElementUV', [['I', 0]], [
        N('Version', [['I', 101]]), N('Name', [['S', 'UVMap']]),
        N('MappingInformationType', [['S', 'ByPolygonVertex']]),
        N('ReferenceInformationType', [['S', 'IndexToDirect']]),
        N('UV', [['d', [0, 0, 1, 0, 1, 1, 0, 1]]]),
        N('UVIndex', [['i', [0, 1, 2, 3]]]),
      ]),
      N('LayerElementMaterial', [['I', 0]], [
        N('Version', [['I', 101]]), N('Name', [['S', '']]),
        N('MappingInformationType', [['S', 'AllSame']]),
        N('ReferenceInformationType', [['S', 'IndexToDirect']]),
        N('Materials', [['i', [0]]]),
      ]),
      N('Layer', [['I', 0]], [
        N('Version', [['I', 100]]),
        N('LayerElement', [], [N('Type', [['S', 'LayerElementNormal']]), N('TypedIndex', [['I', 0]])]),
        N('LayerElement', [], [N('Type', [['S', 'LayerElementMaterial']]), N('TypedIndex', [['I', 0]])]),
        N('LayerElement', [], [N('Type', [['S', 'LayerElementUV']]), N('TypedIndex', [['I', 0]])]),
      ]),
    ]),
    N('Model', [['L', 2345678], ['S', 'quad' + SEP + 'Model'], ['S', 'Mesh']], [
      N('Version', [['I', 232]]),
      N('Properties70', [], [
        N('P', [['S', 'InheritType'], ['S', 'enum'], ['S', ''], ['S', ''], ['I', 1]]),
        N('P', [['S', 'Lcl Translation'], ['S', 'Lcl Translation'], ['S', ''], ['S', 'A'], ['D', 0], ['D', 0], ['D', 0]]),
        N('P', [['S', 'Lcl Rotation'], ['S', 'Lcl Rotation'], ['S', ''], ['S', 'A'], ['D', 0], ['D', 0], ['D', 0]]),
        N('P', [['S', 'Lcl Scaling'], ['S', 'Lcl Scaling'], ['S', ''], ['S', 'A'], ['D', 1], ['D', 1], ['D', 1]]),
        N('P', [['S', 'currentUVSet'], ['S', 'KString'], ['S', ''], ['S', 'U'], ['S', 'UVMap']]),
      ]),
    ]),
    N('Material', [['L', 3456789], ['S', 'mat' + SEP + 'Material'], ['S', '']], [
      N('Version', [['I', 102]]),
      N('ShadingModel', [['S', 'phong']]),
      N('MultiLayer', [['I', 0]]),
      N('Properties70', [], [
        N('P', [['S', 'DiffuseColor'], ['S', 'Color'], ['S', ''], ['S', 'A'], ['D', 1], ['D', 1], ['D', 1]]),
      ]),
    ]),
    N('Texture', [['L', 4567890], ['S', tex + SEP + 'Texture'], ['S', '']], [
      N('Type', [['S', 'TextureVideoClip']]),
      N('Version', [['I', 202]]),
      N('TextureName', [['S', tex + SEP + 'Texture']]),
      N('Media', [['S', tex + SEP + 'Video']]),
      N('FileName', [['S', tex]]),
      N('RelativeFilename', [['S', tex]]),
    ]),
    N('Video', [['L', 4567891], ['S', tex + SEP + 'Video'], ['S', 'Clip']], [
      N('Type', [['S', 'Clip']]),
      N('Properties70', [], [
        N('P', [['S', 'Path'], ['S', 'KString'], ['S', 'XRefUrl'], ['S', ''], ['S', tex]]),
      ]),
      N('UseMipMap', [['I', 0]]),
      N('FileName', [['S', tex]]),
      N('RelativeFilename', [['S', tex]]),
    ]),
  ]),
  N('Connections', [], [
    N('C', [['S', 'OO'], ['L', 2345678], ['L', 0]]),
    N('C', [['S', 'OO'], ['L', 1234567], ['L', 2345678]]),
    N('C', [['S', 'OO'], ['L', 3456789], ['L', 2345678]]),
    N('C', [['S', 'OP'], ['L', 4567890], ['L', 3456789], ['S', 'DiffuseColor']]),
    N('C', [['S', 'OO'], ['L', 4567891], ['L', 4567890]]),
  ]),
  N('Takes', [], [N('Current', [['S', '']])]),
];

push(Buffer.from('Kaydara FBX Binary  \x00', 'latin1')); u8(0x1a); u8(0x00); u32(7400);
let off = 27;
for (const n of tree) { writeNode(n, off); off += nodeSize(n); }
push(Buffer.alloc(13));
while ((len % 16) !== 0) u8(0);
push(Buffer.alloc(4)); u32(7400); push(Buffer.alloc(120));
push(Buffer.from([0xfa, 0xbc, 0xab, 0x09, 0xd0, 0xc8, 0xd4, 0x66, 0xb1, 0x76, 0xfb, 0x83, 0x1c, 0xf7, 0x26, 0x7e]));
push(Buffer.alloc(4));
fs.writeFileSync(path.join(outDir, 'sample_tex.fbx'), Buffer.concat(out));
console.log('generated sample_tex.fbx + texture.png in', outDir);
