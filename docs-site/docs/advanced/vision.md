---
sidebar_position: 6
---

# Vision

## Image to Text with LLM Client
Convert images to text using LLM Client's vision capabilities. This feature allows you to extract information from images, making it useful for various applications like document processing, data extraction, and more.

Example usage:

Using remote images:
```bash
lc -m openai:gpt-4.1-mini -i https://wallpaperaccess.com/full/2556395.jpg "Describe this image"

output:
The image shows a clear blue sky with several seagulls flying. There are at least eight seagulls visible, with two prominent ones in the foreground, their wings spread wide as they glide. The background features a few scattered white clouds near the horizon, adding depth to the sky. The overall scene conveys a sense of freedom and tranquility.
```

Using local images:
```bash
lc -m hyperbolic:mistralai/Pixtral-12B-2409 -i ~/Downloads/490164.jpg "Describe this image"

output:
The image captures a stunning landscape during sunset. The foreground features vibrant pink flowers, possibly bougainvillea, which add a burst of color to the scene. The sky is a beautiful gradient of colors, transitioning from soft pastel hues near the horizon to deeper shades of blue and purple as it stretches upward. The sun is setting behind a range of rolling hills or mountains, casting a warm, golden glow that reflects across the landscape. The trees in the image are silhouetted against the vibrant sky, adding a sense of depth and tranquility to the scene. The overall mood of the image is serene and picturesque, highlighting the natural beauty of the setting.
```
