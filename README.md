# LLM Debate Orchestrator (Quorum)

A Rust-based orchestration system that facilitates structured debates between multiple Large Language Models (LLMs). This tool enables different AI models to engage in multi-round discussions on specific topics, with optional web search capabilities for fact-checking and research.

## Overview

The LLM Debate Orchestrator creates a structured environment where multiple LLMs can participate in a collaborative discussion. Each participant can have unique characteristics (model, temperature, perspective) and optionally perform web searches to support their arguments. The system manages turn-taking, token limits, conversation history, and exports the entire debate to a formatted markdown document.

## Architecture

### Component Overview

```
quorum/
├── src/
│   ├── main.rs           # CLI entry point and argument parsing
│   ├── config.rs         # Configuration structures and validation
│   ├── models.rs         # API request/response data models
│   ├── orchestrator.rs   # Core debate orchestration logic
│   ├── llm_client.rs     # OpenRouter API client
│   ├── search_client.rs  # SearXNG search integration
│   └── markdown.rs       # Markdown export functionality
├── Cargo.toml            # Rust dependencies
└── config.json           # Debate configuration (user-created)
```

### Core Components

#### 1. Main (`main.rs`)
- **Purpose**: Entry point for the CLI application
- **Responsibilities**:
  - Parse command-line arguments (config file path, output path, verbosity)
  - Load and deserialize configuration from JSON
  - Initialize and run the debate orchestrator
  - Export results to markdown
  - Display final statistics

#### 2. Configuration (`config.rs`)
- **Purpose**: Define and validate debate parameters
- **Key Structures**:
  - `Config`: Top-level configuration containing API keys, participants, topic, and rules
  - `DebateRules`: Constraints for the debate (token limits, rounds, search settings)
  - `Participant`: Individual LLM configuration (name, model, system prompt, temperature)
- **Validation**: Ensures all required fields are present and values are within acceptable ranges

#### 3. Models (`models.rs`)
- **Purpose**: Data structures for API communication
- **Key Structures**:
  - `Message`: Conversation history entries (role + content)
  - `OpenRouterRequest`/`OpenRouterResponse`: API request/response formats
  - `SearXNGResponse`/`SearchResult`: Search API responses
- **Design**: Aligned with OpenRouter API specification for seamless integration

#### 4. Orchestrator (`orchestrator.rs`)
- **Purpose**: Central coordination of the debate process
- **Key Responsibilities**:
  - Manage conversation history and state
  - Enforce token limits and round progression
  - Coordinate participant turns
  - Track token usage across the debate
  - Provide user feedback (progress bars, formatted output)
- **Token Management**: Uses `tiktoken-rs` for accurate token counting

#### 5. LLM Client (`llm_client.rs`)
- **Purpose**: Interface with OpenRouter API for LLM responses
- **Key Features**:
  - Sends conversation context to specified LLM models
  - Detects search requests in LLM responses (via `SEARCH:` prefix)
  - Executes searches and provides results back to the LLM
  - Handles two-phase responses: initial + search-augmented
- **Search Pattern**: Models include `SEARCH: <query>` in their responses to request information

#### 6. Search Client (`search_client.rs`)
- **Purpose**: Interface with SearXNG for web searches
- **Functionality**:
  - Queries SearXNG instance with search terms
  - Formats search results for LLM consumption
  - Truncates content to prevent context overflow
  - Returns top 3 results per query

#### 7. Markdown Exporter (`markdown.rs`)
- **Purpose**: Generate formatted debate transcripts
- **Output Includes**:
  - Metadata (topic, date, token usage)
  - Participant details (model, temperature)
  - Context and debate rules
  - Structured transcript with rounds and speakers

## How It Works

### Initialization Flow

```
1. Parse CLI arguments → config.json path, output path, verbose flag
2. Load configuration → deserialize JSON into Config struct
3. Validate configuration → check API keys, participants, rules
4. Initialize LLM Client → with OpenRouter API key and SearXNG URL
5. Initialize conversation history → with system message containing rules and context
6. Create DebateOrchestrator → ready to run debate
```

### Debate Execution Flow

```
For each round (1 to N):
  ├── Print round header
  └── For each participant:
      ├── Check token limit → exit early if approaching max
      ├── Prepare messages → clone history + add participant's system prompt
      ├── Call LLM API → send conversation context
      ├── Process response:
      │   ├── If search enabled:
      │   │   ├── Extract SEARCH: queries from response
      │   │   ├── Execute searches via SearXNG
      │   │   ├── Make follow-up API call with search results
      │   │   └── Return augmented response
      │   └── Else: return initial response
      ├── Add response to conversation history
      ├── Update token count
      └── Display response with formatting
```

### Search Integration

When search is enabled, LLMs can request information by including lines like:
```
SEARCH: climate change statistics 2024
SEARCH: renewable energy adoption rates
```

The system:
1. Extracts all `SEARCH:` prefixed lines from the response
2. Executes each search (up to the configured limit)
3. Formats results from SearXNG
4. Sends results back to the LLM with a prompt to provide the final response
5. Returns the search-augmented response to the debate

### Token Management

The system enforces token limits at two levels:

1. **Per-Turn Limit**: Each participant's response is capped at `max_tokens_per_turn`
2. **Total Limit**: Conversation history tokens + next turn tokens ≤ `max_total_tokens`
   - Uses `tiktoken-rs` with `p50k_base` encoding for accurate counting
   - Exits debate early if approaching the total limit

### Conversation Context

Each participant receives:
- **System message**: Debate rules, topic, context, and search instructions (if enabled)
- **Participant-specific system prompt**: Their role or perspective (if configured)
- **Full conversation history**: All previous turns from all participants

This creates a truly conversational environment where models build upon each other's responses.

## Configuration

### Example `config.json`

```json
{
  "openrouter_api_key": "sk-or-v1-...",
  "searxng_url": "http://localhost:8888",
  "topic": "The Impact of Artificial Intelligence on Employment",
  "context": "Discuss both opportunities and challenges of AI in the workforce, considering short-term disruptions and long-term transformations.",
  "debate_rules": {
    "max_total_tokens": 50000,
    "max_tokens_per_turn": 1000,
    "rounds": 3,
    "enable_search": true,
    "search_limit_per_turn": 2
  },
  "participants": [
    {
      "name": "Optimist",
      "model": "anthropic/claude-3-5-sonnet",
      "system_prompt": "You are an optimistic futurist who believes AI will create more opportunities than it displaces. Support your arguments with data and examples.",
      "temperature": 0.7
    },
    {
      "name": "Pragmatist",
      "model": "openai/gpt-4-turbo",
      "system_prompt": "You are a pragmatic analyst who considers both benefits and challenges of AI. Focus on realistic, balanced assessments.",
      "temperature": 0.5
    },
    {
      "name": "Skeptic",
      "model": "google/gemini-pro-1.5",
      "system_prompt": "You are a critical thinker concerned about the societal impacts of rapid AI adoption. Highlight risks and advocate for thoughtful regulation.",
      "temperature": 0.6
    }
  ]
}
```

### Configuration Fields

#### Top Level
- `openrouter_api_key`: Your OpenRouter API key (required)
- `searxng_url`: Base URL of your SearXNG instance (e.g., "http://localhost:8888")
- `topic`: The main question or topic for debate
- `context`: Additional context, framing, or constraints for the discussion

#### Debate Rules
- `max_total_tokens`: Total token budget for the entire debate
- `max_tokens_per_turn`: Maximum tokens per individual response
- `rounds`: Number of complete rounds (each participant speaks once per round)
- `enable_search`: Whether participants can perform web searches
- `search_limit_per_turn`: Maximum number of searches per participant turn

#### Participants (array)
- `name`: Display name for the participant
- `model`: OpenRouter model identifier (e.g., "anthropic/claude-3-5-sonnet")
- `system_prompt`: Optional perspective or role definition
- `temperature`: Response randomness (0.0 = deterministic, 2.0 = very random)

## Usage

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs)
2. **OpenRouter API Key**: Sign up at [openrouter.ai](https://openrouter.ai)
3. **SearXNG** (optional): For web search functionality
   - Self-hosted: [SearXNG installation guide](https://docs.searxng.org/admin/installation.html)
   - Docker: `docker run -d -p 8888:8080 searxng/searxng`

### Building

```bash
cargo build --release
```

### Running

```bash
# Basic usage (uses config.json in current directory)
cargo run --release

# Specify custom config file
cargo run --release -- --config my_debate_config.json

# Specify output file
cargo run --release -- --output results/debate_2024.md

# Enable verbose mode (shows search queries)
cargo run --release -- --verbose

# All options combined
cargo run --release -- \
  --config configs/ai_employment.json \
  --output results/ai_debate_$(date +%Y%m%d).md \
  --verbose
```

### Command-Line Arguments

- `-c, --config <PATH>`: Path to configuration file (default: `config.json`)
- `-o, --output <PATH>`: Path for markdown output (default: `debate_output.md`)
- `-v, --verbose`: Enable verbose output (shows search queries and debug info)

## Output

### Console Output

The system provides rich console feedback during execution:
- Debate header with topic, participants, and rounds
- Progress bar showing completion status
- Real-time participant responses with formatting
- Token usage and timing information per response
- Final statistics (total tokens used)

### Markdown Export

The generated markdown file includes:
- **Header**: Topic, date, total tokens used
- **Participants**: List of models and their configurations
- **Context**: The debate framing and context
- **Transcript**: Structured by rounds, with each participant's contributions
- **Formatting**: Search results are blockquoted, responses are clearly attributed

## Key Features

### 1. Multi-Model Support
Use different LLM providers in the same debate (Claude, GPT-4, Gemini, etc.) via OpenRouter

### 2. Perspective Diversity
Assign unique system prompts to create diverse viewpoints and roles

### 3. Web Search Integration
Enable fact-checking and research during the debate via SearXNG

### 4. Token Management
Automatic token counting and budget enforcement prevents runaway costs

### 5. Conversation Memory
Full context preservation ensures coherent multi-turn discussions

### 6. Progress Tracking
Real-time progress bars and status updates

### 7. Structured Export
Professional markdown output suitable for documentation or analysis

## Technical Details

### Dependencies

- **tokio**: Async runtime for concurrent operations
- **reqwest**: HTTP client for API communication
- **serde/serde_json**: Configuration and API serialization
- **clap**: Command-line argument parsing
- **anyhow**: Error handling and propagation
- **chrono**: Timestamp generation for exports
- **colored**: Terminal output formatting
- **indicatif**: Progress bar UI
- **tiktoken-rs**: OpenAI-compatible token counting

### API Integration

**OpenRouter**: Unified API for multiple LLM providers
- Endpoint: `https://openrouter.ai/api/v1/chat/completions`
- Format: OpenAI-compatible chat completion API
- Authentication: Bearer token in Authorization header

**SearXNG**: Privacy-respecting metasearch engine
- Endpoint: `{base_url}/search?q={query}&format=json`
- Returns: JSON array of search results with title, URL, and content snippet

### Error Handling

The system uses `anyhow::Result` for comprehensive error propagation:
- Configuration validation errors
- API communication failures
- Token limit violations
- File I/O errors

All errors are descriptive and guide users toward resolution.

## Use Cases

### 1. AI-Assisted Brainstorming
Multiple perspectives on complex problems or decisions

### 2. Research Synthesis
Combine different models' knowledge with web search for comprehensive analysis

### 3. Debate Simulation
Explore multiple sides of controversial or nuanced topics

### 4. Model Comparison
Observe how different LLMs approach the same prompt with identical context

### 5. Content Generation
Create rich, multi-perspective content for articles, reports, or educational materials

### 6. Decision Analysis
Evaluate options from multiple analytical angles with fact-checking

## Limitations and Considerations

1. **Cost**: Multiple LLM API calls can incur significant costs; monitor your token limits
2. **Rate Limits**: OpenRouter may have rate limits depending on your plan
3. **Search Quality**: Depends on SearXNG configuration and available search engines
4. **Context Windows**: Very long debates may exceed some models' context windows
5. **Consistency**: LLM responses are non-deterministic (except at temperature 0.0)

## Future Enhancements

Potential improvements for future versions:
- Support for direct tool calling (instead of SEARCH: prefix parsing)
- Voting/consensus mechanisms for decision-making
- Dynamic participant addition/removal mid-debate
- Real-time web UI for monitoring debates
- Cost estimation and tracking per participant
- Support for image/multimodal inputs
- Debate templates and presets
- Export to additional formats (HTML, PDF, JSON)

## Contributing

Contributions are welcome! Areas for improvement:
- Additional search providers (Google, Bing, etc.)
- Alternative LLM API integrations
- Enhanced markdown formatting
- Performance optimizations
- Test coverage

## License

[Add your license here]

## Acknowledgments

- Built with Rust 🦀
- Powered by [OpenRouter](https://openrouter.ai)
- Search via [SearXNG](https://searxng.org)
- Inspired by the potential of multi-agent AI systems
