---
id: vertex-ai
title: Vertex AI
sidbar_position: 4
---

# Vertex AI
Install the vertex google config using below
```bash
lc providers install vertex_google
```
Add necessary variables - location and project id
```bash
lc providers vars vertex_google set location global
lc p vars vertex_google s project your-project-id
```
Then add your service account details, when prompted for password, give the path to service account json (ex: /path/to/sa.json)
```bash
lc keys add vertex_google
```
Test your configuration
```bash
lc -m vertex_google:gemini-2.5-pro "what is 2+2?"                     
Thinking...📊 Token usage: 111 input + 8 output = 119 total
2 + 2 = 4.
```


