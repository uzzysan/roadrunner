const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const projectRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(projectRoot, '..');
const tokens = require(path.join(repoRoot, 'design/tokens.json'));
const pl = require(path.join(projectRoot, 'src/i18n/locales/pl.json'));
const en = require(path.join(projectRoot, 'src/i18n/locales/en.json'));

const requiredKeys = [
  'auth.alreadyHaveAccount',
  'auth.createAccount',
  'auth.dontHaveAccount',
  'auth.loginButton',
  'auth.phone',
  'auth.registerButton',
  'auth.verify',
  'errors.error',
  'errors.invalidCredentials',
  'errors.somethingWentWrong',
  'navigation.home',
  'navigation.map',
  'navigation.profile',
  'navigation.tickets',
  'validation.emailRequired',
  'validation.passwordMatch',
];

const atPath = (object, dottedPath) => dottedPath
  .split('.')
  .reduce((value, key) => value?.[key], object);

const luminance = (hex) => {
  const channels = hex.slice(1).match(/.{2}/g).map((part) => parseInt(part, 16) / 255);
  const linear = channels.map((value) => value <= 0.04045
    ? value / 12.92
    : ((value + 0.055) / 1.055) ** 2.4);
  return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
};

const contrast = (foreground, background) => {
  const a = luminance(foreground);
  const b = luminance(background);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
};

test('all required runtime translation keys exist in Polish and English', () => {
  for (const key of requiredKeys) {
    assert.equal(typeof atPath(pl, key), 'string', `Missing PL key: ${key}`);
    assert.equal(typeof atPath(en, key), 'string', `Missing EN key: ${key}`);
  }
});

test('all declared contrast pairs pass in both canonical themes', () => {
  for (const scheme of ['light', 'dark']) {
    for (const pair of tokens.contrastPairs) {
      const actual = contrast(
        tokens.color[scheme][pair.foreground],
        tokens.color[scheme][pair.background],
      );
      assert.ok(
        actual >= pair.minimum,
        `${scheme}: ${pair.foreground}/${pair.background} ${actual} < ${pair.minimum}`,
      );
    }
  }
});

test('mobile adapter imports the canonical token source', () => {
  const adapter = fs.readFileSync(path.join(projectRoot, 'src/theme/tokens.ts'), 'utf8');
  assert.match(adapter, /design\/tokens\.json/);
  assert.equal(tokens.version, '2.0.0');
});

test('launcher assets are generated from approved brand sources', () => {
  for (const name of ['app-icon.png', 'adaptive-foreground.png', 'splash.png', 'favicon.png']) {
    assert.ok(fs.existsSync(path.join(projectRoot, 'assets/brand', name)), `Missing ${name}`);
  }
});
