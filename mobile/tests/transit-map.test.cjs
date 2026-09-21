const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');
const ts = require('typescript');

function loadScreen(name, imports = {}) {
  const file = path.join(__dirname, '..', 'src', 'screens', name);
  const source = fs.readFileSync(file, 'utf8');
  const javascript = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.React, target: ts.ScriptTarget.ES2020 } }).outputText;
  const exports = {};
  const context = { exports, URL, console, require(id) {
    if (id in imports) return imports[id];
    if (id === 'react-native') return { StyleSheet: { create: value => value } };
    return {};
  } };
  vm.runInNewContext(javascript, context, { filename: name });
  return { source, exports };
}

const map = loadScreen('MapScreen.tsx');

test('bridge serializes API text as data, never HTML source', () => {
  const hostile = '</script><img src=x onerror=alert(1)>\u2028';
  const serialized = map.exports.serializeWebViewMessage({ type: 'setStops', stops: [{ name: hostile, latitude: Infinity, __proto__: null }] });
  assert.equal(JSON.parse(serialized).stops[0].name, hostile);
  assert.equal(JSON.parse(serialized).stops[0].latitude, null);
  assert.match(map.exports.MAP_HTML_TEMPLATE, /title\.textContent=text\(stop\.name\)/);
  assert.doesNotMatch(map.exports.MAP_HTML_TEMPLATE, /\$\{stop\./);
});

test('navigation is limited to the local document and exact OSM attribution', () => {
  const kind = map.exports.mapNavigationKind;
  assert.equal(kind('about:blank'), 'internal');
  assert.equal(kind('https://app.roadrunner.invalid/'), 'internal');
  assert.equal(kind('https://www.openstreetmap.org/copyright'), 'attribution');
  for (const url of ['https://www.openstreetmap.org.evil.test/copyright', 'http://www.openstreetmap.org/copyright', 'https://www.openstreetmap.org/evil', 'javascript:alert(1)', 'not a url']) assert.equal(kind(url), 'blocked', url);
});

test('OSM attribution, constrained WebView and non-danger selected stop stay explicit', () => {
  assert.match(map.exports.MAP_HTML_TEMPLATE, /<meta name="viewport" content="width=device-width,initial-scale=1"/);
  assert.doesNotMatch(map.exports.MAP_HTML_TEMPLATE, /maximum-scale|user-scalable=no/i);
  assert.match(map.exports.MAP_HTML_TEMPLATE, /attributionControl:true/);
  assert.match(map.exports.MAP_HTML_TEMPLATE, /OpenStreetMap<\/a> contributors/);
  assert.match(map.source, /originWhitelist=\{\['about:blank', 'https:\/\/app\.roadrunner\.invalid'\]\}/);
  assert.match(map.exports.MAP_HTML_TEMPLATE, /\.stop-marker\.selected\{[^}]*outline:4px/);
  assert.equal(map.exports.getSelectedStopPalette({ primary: '#006B63', onPrimary: '#FFFFFF' }).fill, '#006B63');
});

test('API line colours receive contrast-safe text or semantic fallback', () => {
  const pick = map.exports.getAccessibleLineColor;
  assert.equal(pick('red; background:url(x)', '#D7F5ED', '#102C36', '#FFFFFF', '#005249').background, '#D7F5ED');
  const palette = pick('#000000', '#D7F5ED', '#102C36', '#FFFFFF', '#005249');
  assert.equal(palette.background, '#000000');
  assert.equal(palette.foreground, '#FFFFFF');
});

test('stop details reuses the audited bridge and validates directions coordinates', () => {
  const stop = loadScreen('StopDetailsScreen.tsx', { './MapScreen': map.exports });
  assert.equal(stop.exports.directionsUrl(NaN, 21), null);
  assert.equal(stop.exports.directionsUrl(100, 21), null);
  assert.match(stop.exports.directionsUrl(52.2297, 21.0122), /^https:\/\/www\.openstreetmap\.org\/\?mlat=52\.229700&mlon=21\.012200/);
  assert.match(stop.source, /MAP_HTML_TEMPLATE/);
  assert.doesNotMatch(stop.source, /originWhitelist=\{\['\*'\]\}/);
});

test('map and route retain list access, safe popup text and visible provenance', () => {
  const route = loadScreen('RouteDetailsScreen.tsx', { './MapScreen': map.exports });
  assert.match(map.source, /\['map', 'list'\]/);
  assert.match(map.source, /visibleStops\.map\(/);
  assert.match(map.source, /navigation\.navigate\('StopDetails'/);
  assert.match(route.exports.ROUTE_MAP_HTML, /popup\.textContent=stop\.name/);
  assert.match(route.exports.ROUTE_MAP_HTML, /attributionControl:true/);
  assert.match(route.source, /transit_common\.scheduled/);
  assert.match(route.source, /transit_common\.retrieved_at/);
});
