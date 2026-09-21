const path = require('node:path');
const { getDefaultConfig } = require('expo/metro-config');

const projectRoot = __dirname;
const repositoryRoot = path.resolve(projectRoot, '..');
const config = getDefaultConfig(projectRoot);

// The canonical design token file is shared by every client from the repo root.
config.watchFolders = [...new Set([...(config.watchFolders ?? []), repositoryRoot])];

module.exports = config;
