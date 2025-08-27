---
id: gemini
title: Gemini
sidebar_position: 3
---

# Gemini
To install Gemini use below
```bash
lc providers install gemini
```
Add your key
```bash
lc keys add gemini
```
## Test your configuration
### To list models
```
lc p m gemini
```
### Text generation
```
lc -m gemini:gemini-2.5-pro "what is 2+2?" --max-tokens 5k
```
### Image generation
```
lc img -m gemini:gemini-2.5-pro  "A small robot in a garden" --output ~/Downloads
```
### Image Understanding
```
lc -m gemini:gemini-1.5-flash "What do you see in this image?" --image "test_images/image_20250825_023446_1.png"
```
### Embeddings
```
lc embed -m gemini:gemini-embedding-001 "test text"
```