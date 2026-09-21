const fs = require('node:fs/promises');
const path = require('node:path');
const sharp = require('sharp');

const mobileRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(mobileRoot, '..');
const brandSource = path.join(repoRoot, 'assets/brand');
const output = path.join(mobileRoot, 'assets/brand');

async function centered(source, sourceSize, canvasSize, destination) {
  const inset = Math.floor((canvasSize - sourceSize) / 2);
  const image = await sharp(source).resize(sourceSize, sourceSize).png().toBuffer();
  await sharp({
    create: {
      width: canvasSize,
      height: canvasSize,
      channels: 4,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    },
  }).composite([{ input: image, left: inset, top: inset }]).png().toFile(destination);
}

async function generate() {
  await fs.mkdir(output, { recursive: true });
  await Promise.all([
    sharp(path.join(brandSource, 'roadrunner-app-icon.svg'))
      .resize(1024, 1024)
      .png()
      .toFile(path.join(output, 'app-icon.png')),
    sharp(path.join(brandSource, 'roadrunner-app-icon.svg'))
      .resize(64, 64)
      .png()
      .toFile(path.join(output, 'favicon.png')),
    sharp(path.join(brandSource, 'roadrunner-lockup-light.svg'))
      .resize({ width: 728 })
      .png()
      .toFile(path.join(output, 'lockup-light.png')),
    sharp(path.join(brandSource, 'roadrunner-lockup-dark.svg'))
      .resize({ width: 728 })
      .png()
      .toFile(path.join(output, 'lockup-dark.png')),
    sharp(path.join(brandSource, 'roadrunner-mark-light.svg'))
      .resize(256, 256)
      .png()
      .toFile(path.join(output, 'mark-light.png')),
    sharp(path.join(brandSource, 'roadrunner-mark-dark.svg'))
      .resize(256, 256)
      .png()
      .toFile(path.join(output, 'mark-dark.png')),
    centered(
      path.join(brandSource, 'roadrunner-mark-dark.svg'),
      384,
      1024,
      path.join(output, 'adaptive-foreground.png'),
    ),
    centered(
      path.join(brandSource, 'roadrunner-mark-dark.svg'),
      256,
      512,
      path.join(output, 'splash.png'),
    ),
  ]);
}

generate().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
