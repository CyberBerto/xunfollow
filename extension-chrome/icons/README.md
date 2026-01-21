# Icons

Place your extension icons here:

- `icon16.png` - 16x16 pixels
- `icon48.png` - 48x48 pixels
- `icon128.png` - 128x128 pixels

## Quick Icon Generation

You can use any of these methods:

### Option 1: Generate online
Visit https://icon.kitchen/ and upload a logo to generate all sizes

### Option 2: Use ImageMagick
```bash
# If you have a source image (logo.png)
convert logo.png -resize 16x16 icon16.png
convert logo.png -resize 48x48 icon48.png
convert logo.png -resize 128x128 icon128.png
```

### Option 3: Use placeholder
For testing, create simple colored squares:
```bash
convert -size 16x16 xc:"#1da1f2" icon16.png
convert -size 48x48 xc:"#1da1f2" icon48.png
convert -size 128x128 xc:"#1da1f2" icon128.png
```

The extension will work without icons, but Chrome will show a default icon instead.
