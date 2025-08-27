---
id: openai
title: Open AI
sidbar_position: 1
---

# Open AI
To  install the Open AI use below
```bash
lc providers install openai
```
Add your key
```bash
lc keys add openai
```
## Test your configuration
### To list all models
```
lc p m openai
```
### Text generation
```
lc -m openai:gpt-4o-mini 'What is 2+2?'
lc -m openai:gpt-4o-mini --stream 'Count from 1 to 5'
```
### Image generation
```
lc img -m openai:dall-e-3 "A futuristic city with flying cars" --output /tmp
lc img -m openai:gpt-image-1 "A futuristic city with flying cars"
```
### Image understanding
```
lc -m openai:gpt-4o-mini -i https://wallpaperaccess.com/full/2556395.jpg "Describe this image"
```
### Text embedding
```
lc embed -m openai:text-embedding-3-small "test text"
```

