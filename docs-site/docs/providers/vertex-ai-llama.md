---
id: vertex-ai-llama
title: Vertex AI (llama models)
sidbar_position: 5
---

# Vertex AI
Install the vertex ai llama config using below
```bash
lc providers install vertex_llama
```
Add necessary variables - location and project id
```bash
lc providers vars vertex_llama set location global
lc p vars vertex_llama s project your-project-id
```
Then add your service account details, when prompted for password, give the path to service account json (ex: /path/to/sa.json)
```bash
lc keys add vertex_llama
```
Test your configuration
```bash
lc -m vertex_llama:meta/llama-4-maverick-17b-128e-instruct-maas "what is 2+2?"
Thinking...📊 Token usage: 111 input + 8 output = 119 total
2 + 2 = 4.
```