# Crux Agent

Crux Agent is a cross-platform AI voice agent designed to run on multiple devices, including PCs, phones, and tablets. The goal is to create an open agent system where users can contribute and share function calls, effectively building a decentralized, community-driven AI assistant akin to a home Jarvis.

## Features
- Cross-platform support (Android, iOS, PC, etc.)
- Uses [LiveKit](https://github.com/livekit/livekit) for real-time communication
- Business logic powered by [Crux](https://github.com/redbadger/crux)
- AI processing via:
  - [Kokoro-FastAPI](https://github.com/remsky/Kokoro-FastAPI)
  - [OpenedAI-Whisper](https://github.com/matatonic/openedai-whisper)
  - [Ollama](https://github.com/ollama/ollama)
- Open function call system for community contributions

## Getting Started

### Prerequisites
To run Crux Agent locally, you'll need:

- A local [LiveKit server](https://github.com/livekit/livekit)
- Rust toolchain installed
- Dependencies listed above

### Build Instructions

1. Clone the repository:
   ```sh
   git clone https://github.com/fraschm1998/crux-agent.git
   cd crux-agent
   ```

2. Compile with `uniffi-bindgen` feature:
   ```sh
   cargo build --package shared --features uniffi-bindgen
   ```

3. Run the application (More details coming soon).

## Roadmap
- Improve documentation with AI assistance
- Extend platform support
- Develop a plugin system for custom function calls
- Optimize AI processing for real-time interaction

## Credits
A huge thanks to the following projects for making Crux Agent possible:
- [Crux](https://github.com/redbadger/crux) for their amazing business logic framework
- [LiveKit](https://github.com/livekit/livekit) for real-time communication
- [Kokoro-FastAPI](https://github.com/remsky/Kokoro-FastAPI) for AI processing
- [OpenedAI-Whisper](https://github.com/matatonic/openedai-whisper) for speech recognition
- [Ollama](https://github.com/ollama/ollama) for additional AI capabilities

## Contributing
This is an open project! If you’d like to contribute function calls or improve the agent, feel free to fork, submit PRs, or join the discussion.

## License
[MIT License](LICENSE)

---
More details and updates coming soon!
