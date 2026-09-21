/* eslint-disable @typescript-eslint/no-var-requires -- Node's built-in test runner loads this CommonJS file. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');
const ts = require('typescript');

const scannerPath = path.join(__dirname, '../src/screens/QRScannerScreen.tsx');
const source = fs.readFileSync(scannerPath, 'utf8');
const sourceFile = ts.createSourceFile(scannerPath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
const names = new Set(['classifyValidationPayload', 'classifyValidationFailure']);
const functions = sourceFile.statements
  .filter((statement) => ts.isFunctionDeclaration(statement) && names.has(statement.name?.text))
  .map((statement) => statement.getText(sourceFile).replace(/^export\s+/, ''));

assert.equal(functions.length, names.size, 'Scanner classification functions must remain testable');

const javascript = ts.transpileModule(functions.join('\n'), {
  compilerOptions: { target: ts.ScriptTarget.ES2022 },
}).outputText;
const { classifyValidationPayload, classifyValidationFailure } = vm.runInNewContext(
  `${javascript}\n({ classifyValidationPayload, classifyValidationFailure })`,
  { axios: { isAxiosError: (error) => error?.isAxiosError === true } },
);

test('only an explicit negative server verdict marks a ticket invalid', () => {
  assert.equal(classifyValidationPayload({ valid: true }).kind, 'valid');
  assert.equal(classifyValidationPayload({ valid: false }).kind, 'invalid');
  assert.equal(classifyValidationPayload({}).kind, 'unverified');
  assert.equal(classifyValidationPayload(null).kind, 'unverified');
});

test('network and server failures never mark a ticket invalid', () => {
  assert.equal(classifyValidationFailure({ isAxiosError: true }).kind, 'unverified');
  assert.equal(classifyValidationFailure({ isAxiosError: true }).reason, 'offline');
  assert.equal(classifyValidationFailure({ isAxiosError: true, response: { status: 500 } }).reason, 'server');
  assert.equal(classifyValidationFailure(new Error('unexpected')).kind, 'unverified');
});
