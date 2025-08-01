# Logo Setup Guide

Your logo has been copied to `static/img/logo.jpeg` and configured in the documentation site.

## Current Setup

- **Logo location**: `static/img/logo.jpeg`
- **Used as**: Favicon and navbar logo
- **Size in navbar**: 32x32 pixels (auto-resized by browser)

## Recommended Optimizations

For best results, you should create optimized versions of your logo:

### 1. Create an SVG Version (Recommended)

SVG logos scale perfectly at any size. If possible, convert your logo to SVG:

```bash
# Using an online converter like:
# - https://convertio.co/jpg-svg/
# - https://www.adobe.com/express/feature/image/convert/jpg-to-svg

# Then save as static/img/logo.svg
```

### 2. Create Multiple Sizes

For better performance, create these versions:

```bash
# Using ImageMagick (install with: brew install imagemagick)
cd static/img

# Create favicon (16x16 and 32x32)
convert logo.jpeg -resize 16x16 favicon-16.png
convert logo.jpeg -resize 32x32 favicon-32.png

# Create navbar logo (64x64 for retina displays)
convert logo.jpeg -resize 64x64 logo-64.png

# Create social card image (1200x630)
convert logo.jpeg -resize 1200x630 -gravity center -background white -extent 1200x630 social-card.png
```

### 3. Optimize File Size

```bash
# For JPEG (install with: brew install jpegoptim)
jpegoptim --size=50k logo.jpeg

# For PNG (install with: brew install optipng)
optipng -o7 *.png
```

### 4. Create a Favicon.ico

```bash
# Combine multiple sizes into one .ico file
convert favicon-16.png favicon-32.png favicon.ico
```

## Update Configuration

After creating optimized versions, update `docusaurus.config.js`:

```javascript
// For SVG logo
logo: {
  alt: 'LLM Client Logo',
  src: 'img/logo.svg',
  width: 32,
  height: 32,
},

// For favicon.ico
favicon: 'img/favicon.ico',
```

## Alternative: Use an Online Tool

If you don't want to install command-line tools:

1. **Favicon Generator**: https://favicon.io/
   - Upload your JPEG
   - Download the generated files
   - Copy to `static/img/`

2. **Logo Resizer**: https://www.iloveimg.com/resize-image
   - Create different sizes
   - Download optimized versions

3. **SVG Converter**: https://convertio.co/jpg-svg/
   - Convert to SVG if your logo is simple enough

## Current Status

✅ Logo is configured and will work as-is
⚠️ For production, consider creating optimized versions
📱 Current JPEG will be auto-resized by browsers (may affect quality)

The site will work fine with the current JPEG, but optimized versions will improve loading speed and display quality.