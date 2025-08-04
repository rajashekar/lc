---
sidebar_position: 6
---

# Vision


## Text to Image with LLM Client
Generate images from text prompts using LLM Client's vision capabilities. This feature allows you to create visual content based on textual descriptions, useful for creative projects, marketing materials, and more.

Example usage:
```bash
lc img -m openai:dall-e-3 "A futuristic city with flying cars"
🎨 Generating 1 image(s) with prompt: "A futuristic city with flying cars"
🤖 Model: dall-e-3
🏢 Provider: openai
📐 Size: 1024x1024
✅ Successfully generated 1 image(s)!

🖼️ Image 1/1
   URL: https://oaidalleapiprodscus.blob.core.windows.net/private/org-lGnXfLEyZnam3ZCwAsGIi4WT/user-0pB0O9XwbuSBLd2ZjJ5BJbIH/img-c4DYG4ndHY7exw1kFUnxa7la.png?st=2025-08-04T09%3A03%3A17Z&se=2025-08-04T11%3A03%3A17Z&sp=r&sv=2024-08-04&sr=b&rscd=inline&rsct=image/png&skoid=cc612491-d948-4d2e-9821-2683df3719f5&sktid=a48cca56-e6da-484e-a814-9c849652bcb3&skt=2025-08-03T18%3A53%3A20Z&ske=2025-08-04T18%3A53%3A20Z&sks=b&skv=2024-08-04&sig=YLclsnvA%2BT93QZxOq6FQDQJw2fQrukL5ZyHUtFhJLew%3D
   Revised prompt: Depict a visionary glimpse of the future with a sprawling city that stretches across the horizon. In this city of tomorrow, skyscrapers tower high into the clouds, their sleek exteriors glistening under the glow of neon lights. The architecture marries eco-friendly advancements and high tech aesthetics, forming a harmonious blend of green spaces and glass buildings. The sky above is alive with the hum of flying cars, their streamlined bodies darting amongst holographic billboards. Patches of green are visible atop buildings, indicating thriving rooftop gardens. The city is a beacon of prosperity, reflecting mankind's progress in this futuristic utopia.
```


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
