const fs = require('fs');
// stdout: UTF-16LE without BOM (IfcConvert 系出力を再現)
fs.writeSync(1, Buffer.from('[IfcConvert] Processing model.ifc\nDone.\n', 'utf16le'));
// stderr: cp932 encoded Japanese error (PowerShell 日本語エラー相当)
fs.writeSync(2, fs.readFileSync(__dirname + '/verify116_err.bin'));
process.exit(3);
