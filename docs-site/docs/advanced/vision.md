---
sidebar_position: 6
---

# Vision

## Text to Image with LLM Client
Generate images from text prompts using LLM Client's vision capabilities. This feature allows you to create visual content based on textual descriptions, useful for creative projects, marketing materials, and more.

Example usage:
## Dall-E (via openai)
```bash
lc img -m openai:dall-e-3 "A futuristic city with flying cars"

🎨 Generating 1 image(s) with prompt: "A futuristic city with flying cars"
🤖 Model: dall-e-3
🏢 Provider: openai
📐 Size: 1024x1024
✅ Successfully generated 1 image(s)!

🖼️ Image 1/1
   URL: https://oaidalleapiprodscus.blob.core.windows.net/private/org-lGnXfLEyZnam3ZCwAsGIi4WT/user-0pB0O9XwbuSBLd2ZjJ5BJbIH/img-OZ0yhSjeNX3DmS5HJscvbYqV.png?st=2025-08-10T07%3A30%3A37Z&se=2025-08-10T09%3A30%3A37Z&sp=r&sv=2024-08-04&sr=b&rscd=inline&rsct=image/png&skoid=77e5a8ec-6bd1-4477-8afc-16703a64f029&sktid=a48cca56-e6da-484e-a814-9c849652bcb3&skt=2025-08-09T20%3A20%3A26Z&ske=2025-08-10T20%3A20%3A26Z&sks=b&skv=2024-08-04&sig=4DIkbrkimvxn/RYHM1rD159Y8GGG6r3M5XCEidy75HY%3D
   Revised prompt: Imagine a forward-thinking city in the distant future with marvels of engineering and technology. The sky is teeming with a variety of flying cars, from small-scale family vehicles to large public transports, each following their respective lanes of light suspended high above the ground. The high-tech vehicles sport shimmering hues and aerodynamic designs, blending harmoniously with the city's futuristic architecture. Tall, sleek structures of glass and silver tower above the streets, their twinkling lights reflected on the glossy surface of the cars. People of all descents and genders are seen going about their day, highlighting the city's diversity and culture.

💡 Use --output <directory> to automatically download images
```
<p align="center">
<img src="/img/dalle_flying_cars.png" alt="dalle flaying cars" width="500"/>
</p>

To save to folder
```
lc img -m openai:dall-e-3 "A futuristic city with flying cars" --output /tmp
```
you will something like below - 
💾 Saved to: /tmp/image_20250810_083301_1.png

## Flux-dev (via nebius)
```
lc img -m nebius:black-forest-labs/flux-dev "A futuristic city with flying cars" --output /tmp
🎨 Generating 1 image(s) with prompt: "A futuristic city with flying cars"
🤖 Model: black-forest-labs/flux-dev
🏢 Provider: nebius
📐 Size: 1024x1024
✅ Successfully generated 1 image(s)!

🖼️ Image 1/1
   URL: https://pictures-storage.storage.eu-north1.nebius.cloud/text2img-48945751-598d-47e5-b358-e57642dfb8af_00001_.webp
   💾 Saved to: /tmp/image_20250810_085300_1.png
```

<p align="center">
<img src="/img/flux_dev_flying_cars.png" alt="flux dev flaying cars" width="500"/>
</p>

## Flux-dev (via deepinfra)
```
lc img -m deepinfra:black-forest-labs/FLUX-1-dev "A futuristic city with flying cars" --output ~/Downloads
🎨 Generating 1 image(s) with prompt: "A futuristic city with flying cars"
🤖 Model: black-forest-labs/FLUX-1-dev
🏢 Provider: deepinfra
📐 Size: 1024x1024
✅ Successfully generated 1 image(s)!

🖼️ Image 1/1 (Base64)
   💾 Saved to: ~/Downloads/image_20250810_092102_1.png
```

<p align="center">
<img src="/img/flux_dev_deepinfra_flying_cars.png" alt="flux dev flaying cars" width="500"/>
</p>


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
