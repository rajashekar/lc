---
sidebar_position: 18
---

# Audio Commands

The `lc` tool provides comprehensive audio support, enabling both speech-to-text (transcription) and text-to-speech (TTS) capabilities.

## Audio Transcription

Convert audio files to text using the `transcribe` command.

### Basic Usage

```bash
# Transcribe a single audio file
lc transcribe audio.wav

# Transcribe multiple audio files
lc transcribe file1.mp3 file2.wav file3.flac

# Use alias
lc tr recording.mp3
```

### Options

- `-m, --model <MODEL>` - Specify the transcription model (default: whisper-1)
- `-p, --provider <PROVIDER>` - Specify the provider (default: openai)
- `-l, --language <LANG>` - Specify the audio language (e.g., en, es, fr)
- `--prompt <TEXT>` - Provide context to guide the transcription
- `-f, --format <FORMAT>` - Output format: text, json, srt, vtt (default: text)
- `-t, --temperature <TEMP>` - Sampling temperature (0-1, default: 0)

### Supported Audio Formats

- MP3, MP4, MPEG, MPGA, M4A
- WAV
- WebM
- OGG
- FLAC

### Examples

```bash
# Transcribe with language hint
lc transcribe interview.mp3 --language en

# Get JSON output with timestamps
lc transcribe podcast.wav --format json

# Provide context for better accuracy
lc transcribe medical_recording.mp3 --prompt "Medical consultation discussing symptoms"

# Use a different provider
lc transcribe audio.wav --provider custom-provider
```

## Text-to-Speech (TTS)

Convert text to speech using the `tts` command.

### Basic Usage

```bash
# Generate speech from text
lc tts "Hello, this is a test of text to speech"

# Save to a specific file
lc tts "Welcome to our service" --output welcome.mp3

# Read from a file
lc tts --file script.txt --output narration.mp3
```

### Options

- `-m, --model <MODEL>` - TTS model: tts-1, tts-1-hd (default: tts-1)
- `-p, --provider <PROVIDER>` - Specify the provider (default: openai)
- `-v, --voice <VOICE>` - Voice selection: alloy, echo, fable, onyx, nova, shimmer (default: alloy)
- `-o, --output <FILE>` - Output file path (default: speech_TIMESTAMP.mp3)
- `-f, --format <FORMAT>` - Audio format: mp3, opus, aac, flac, wav, pcm (default: mp3)
- `-s, --speed <SPEED>` - Speech speed: 0.25 to 4.0 (default: 1.0)
- `--file <FILE>` - Read text from a file instead of command line

### Voice Options

- **alloy** - Neutral and balanced
- **echo** - Warm and conversational  
- **fable** - Expressive and dynamic
- **onyx** - Deep and authoritative
- **nova** - Friendly and upbeat
- **shimmer** - Soft and pleasant

### Examples

```bash
# Use HD model for better quality
lc tts "Important announcement" --model tts-1-hd

# Generate with different voice
lc tts "Welcome message" --voice nova --output welcome.mp3

# Adjust speech speed
lc tts "Quick instructions" --speed 1.5

# Generate in different format
lc tts "Audio book chapter" --format flac --output chapter1.flac

# Read from file with specific voice
lc tts --file presentation.txt --voice onyx --output presentation.mp3
```

## Audio Attachments in Chat

You can attach audio files to chat prompts for context-aware conversations.

### Basic Usage

```bash
# Ask about audio content
lc "What is being discussed in this recording?" --audio meeting.mp3

# Multiple audio files
lc "Summarize these interviews" --audio interview1.mp3 --audio interview2.wav

# Combine with other attachments
lc "Analyze this presentation" --audio narration.mp3 --image slides.png
```

### How It Works

1. Audio files are automatically transcribed using the Whisper model
2. Transcriptions are added to the chat context
3. The LLM processes both your prompt and the transcribed content

### Examples

```bash
# Meeting transcription and summary
lc "Provide meeting minutes for this recording" --audio meeting_recording.mp3 -m gpt-4o

# Language translation
lc "Translate this Spanish audio to English" --audio spanish_audio.wav

# Content analysis
lc "What are the key points discussed?" --audio podcast_episode.mp3

# Multiple audio analysis
lc "Compare the topics discussed in these recordings" \
  --audio episode1.mp3 \
  --audio episode2.mp3 \
  -m claude-3-opus-20240229
```

## Configuration

### Setting Default Audio Provider

```bash
# Set default provider for audio commands
lc config set audio-provider openai

# Set default transcription model
lc config set audio-model whisper-1

# Set default TTS model
lc config set tts-model tts-1-hd

# Set default TTS voice
lc config set tts-voice nova
```

### Provider Configuration

Audio endpoints can be configured in provider TOML files:

```toml
[provider]
name = "custom-audio"
base_url = "https://api.custom.com/v1"

# Audio transcription endpoint
audio_path = "/audio/transcriptions"

# Text-to-speech endpoint  
speech_path = "/audio/speech"

# Optional: Custom templates for request/response transformation
[audio_templates]
whisper-1 = "custom_whisper_template.tera"

[speech_templates]
tts-1 = "custom_tts_template.tera"
```

## Tips and Best Practices

### Audio Quality
- Use high-quality audio files for better transcription accuracy
- WAV and FLAC formats preserve quality better than compressed formats
- For TTS, use tts-1-hd model for production-quality output

### Performance
- Transcription time depends on audio length and quality
- Batch process multiple files when possible
- Consider using JSON format for transcriptions if you need timestamps

### Language Support
- Whisper supports 50+ languages for transcription
- Always specify language hint for non-English audio
- TTS voices are optimized for English but support multiple languages

### Cost Optimization
- tts-1 is faster and cheaper than tts-1-hd
- Compress audio files before transcription to reduce upload time
- Use appropriate audio formats (MP3 for speech, WAV for music)

## Error Handling

Common issues and solutions:

```bash
# Unsupported format error
# Solution: Convert to supported format first
ffmpeg -i audio.aiff -acodec libmp3lame audio.mp3
lc transcribe audio.mp3

# Large file error
# Solution: Split or compress the audio
ffmpeg -i large_file.wav -t 3600 part1.wav  # First hour
lc transcribe part1.wav

# API rate limits
# Solution: Add delays between requests or batch process
lc transcribe *.mp3 --delay 1
```

## Integration Examples

### Podcast Workflow
```bash
# Transcribe podcast
lc transcribe podcast.mp3 --format srt --output podcast.srt

# Generate summary
lc "Summarize this podcast transcript" --file podcast.srt

# Create promotional audio
lc tts --file summary.txt --voice nova --output promo.mp3
```

### Meeting Assistant
```bash
# Transcribe meeting
lc transcribe meeting.m4a --output meeting.txt

# Extract action items
lc "List all action items from this meeting" --file meeting.txt

# Generate audio summary
lc tts "Meeting summary: $(cat summary.txt)" --output meeting_summary.mp3
```

### Language Learning
```bash
# Transcribe foreign language audio
lc transcribe spanish_lesson.mp3 --language es

# Get translation
lc "Translate this to English and explain grammar" --audio spanish_lesson.mp3

# Generate pronunciation guide
lc tts "Hola, ¿cómo estás?" --voice shimmer --speed 0.8