---
id: anthropic
title: Anthropic
sidebar_position: 2
---

# Anthropic
To install Anthropic use below
```bash
lc providers install anthropic
```
Add your key
```bash
lc keys add anthropic
```
## Test your configuration
### To list models
```
lc p m anthropic
```
### Text generation 
```
lc -m anthropic:claude-sonnet-4-20250514 "write a python program to read json file"
```
### Image understanding
```
lc -m anthropic:claude-sonnet-4-20250514 "Compare this cat to a typical tabby cat. What similarities and differences do you notice?" --image "test_images/image_20250825_023446_1.png"
```