# Phase 4: Log Viewer - Research

**Researched:** 2026-02-28
**Domain:** Docker container log streaming with Tauri v2 + Svelte 5
**Confidence:** HIGH

## Summary

This phase requires streaming Docker container logs (stdout/stderr) in real-time from the Rust backend to the Svelte frontend. The project already has established patterns for event-based streaming (used in image pulling), which can be adapted for log streaming.

**Primary recommendation:** Use bollard's `logs()` stream with `follow: true`, iterate over the stream with `futures::StreamExt`, and emit log lines to the frontend using Tauri's `ipc::Channel` (preferred for streaming) or the existing `app.emit()` pattern.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bollard | 0.16+ | Docker API client | Already in use, provides `logs()` stream |
| tauri::ipc::Channel | v2 | Rust→Frontend streaming | Built-in, optimized for streaming data |
| @tauri-apps/api/event | v2 | Frontend event listening | Already in use for pull-progress |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| futures::StreamExt | 0.3 | Stream iteration | Processing bollard log stream |
| tokio | 1.x | Async runtime | Already configured with full features |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `ipc::Channel` | `app.emit()` | emit() already used in project but Channel is optimized for streaming |
| Custom polling | Docker logs stream | Polling is inefficient and can miss logs |

**No new dependencies required** - all needed crates are already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/
│   ├── logs.rs          # NEW: Log streaming commands
│   └── mod.rs           # Add logs module
└── docker/
    └── client.rs        # Extend with log streaming method

src/lib/components/
└── LogViewer.svelte     # NEW: Log display component
```

### Pattern 1: Bollard Log Streaming (Rust Backend)

**What:** Stream container logs from Docker daemon via bollard's `logs()` method.

**When to use:** All log streaming operations.

**Example:**
```rust
// Source: docs.rs/bollard + project patterns
use bollard::Docker;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::container::LogOutput;
use futures::StreamExt;
use tauri::ipc::Channel;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LogEvent {
    StdOut { message: String },
    StdErr { message: String },
    Error { message: String },
    Eof,
}

#[tauri::command]
pub async fn stream_container_logs(
    container_name: String,
    on_log: Channel<LogEvent>,
    tail: Option<u64>,
) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(true)  // Stream live logs
        .tail(tail.map(|n| n.to_string()).unwrap_or_else(|| "100".to_string()))
        .timestamps(true)
        .build();

    let mut stream = docker.logs(&container_name, Some(options));

    while let Some(result) = stream.next().await {
        match result {
            Ok(log_output) => {
                let event = match log_output {
                    LogOutput::StdOut { message } => LogEvent::StdOut {
                        message: String::from_utf8_lossy(&message).to_string(),
                    },
                    LogOutput::StdErr { message } => LogEvent::StdErr {
                        message: String::from_utf8_lossy(&message).to_string(),
                    },
                    _ => continue, // Skip StdIn/Console
                };
                on_log.send(event).map_err(|e| format!("Channel error: {}", e))?;
            }
            Err(e) => {
                on_log.send(LogEvent::Error {
                    message: format!("Log stream error: {}", e),
                }).ok();
                break;
            }
        }
    }

    on_log.send(LogEvent::Eof).ok();
    Ok(())
}
```

### Pattern 2: Frontend Log Consumer (Svelte 5)

**What:** Listen for log events and display with auto-scroll.

**When to use:** LogViewer component.

**Example:**
```svelte
<script lang="ts">
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { tick } from 'svelte';

  interface LogEvent {
    type: 'StdOut' | 'StdErr' | 'Error' | 'Eof';
    data: { message: string };
  }

  interface Props {
    containerName: string;
    tail?: number;
  }

  let { containerName, tail = 100 }: Props = $props();

  let logs = $state<{ type: string; message: string }[]>([]);
  let logContainer: HTMLDivElement | undefined = $state();
  let isStreaming = $state(false);
  let error = $state('');

  async function startLogStream() {
    isStreaming = true;
    error = '';
    logs = [];

    const onLog = new Channel<LogEvent>();
    
    onLog.onmessage = async (event) => {
      if (event.type === 'Eof') {
        isStreaming = false;
        return;
      }
      
      if (event.type === 'Error') {
        error = event.data.message;
        isStreaming = false;
        return;
      }

      logs = [...logs, { 
        type: event.type, 
        message: event.data.message 
      }];
    };

    try {
      await invoke('stream_container_logs', {
        containerName,
        onLog,
        tail,
      });
    } catch (e) {
      error = String(e);
      isStreaming = false;
    }
  }

  // Auto-scroll when new logs arrive
  $effect.pre(() => {
    logs; // Track logs for reactivity
    tick().then(() => {
      if (logContainer) {
        const autoscroll = 
          logContainer.offsetHeight + logContainer.scrollTop > 
          logContainer.scrollHeight - 50;
        
        if (autoscroll) {
          logContainer.scrollTo(0, logContainer.scrollHeight);
        }
      }
    });
  });

  onMount(startLogStream);
</script>

<div class="log-viewer">
  <div class="log-header">
    <span>Logs: {containerName}</span>
    {#if isStreaming}
      <span class="streaming-indicator">● Live</span>
    {/if}
  </div>
  
  <div bind:this={logContainer} class="log-content">
    {#each logs as log}
      <div class="log-line" class:stderr={log.type === 'StdErr'}>
        {log.message}
      </div>
    {/each}
  </div>
  
  {#if error}
    <div class="log-error">{error}</div>
  {/if}
</div>
```

### Anti-Patterns to Avoid

- **Polling for logs:** Creates unnecessary overhead and can miss logs between polls. Always use `follow: true` for streaming.
- **Not handling UTF-8 errors:** Docker logs may contain non-UTF8 bytes. Use `String::from_utf8_lossy()` to handle gracefully.
- **Blocking the stream:** Don't do heavy processing in the stream loop; emit immediately and let frontend handle display.
- **Not cleaning up listeners:** Always unsubscribe from events when component is destroyed.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Log streaming protocol | Custom WebSocket or polling | bollard `logs()` stream | Handles multiplexed stdout/stderr correctly |
| Real-time frontend updates | Manual state polling | Tauri Channel or events | Optimized IPC, no polling overhead |
| Auto-scroll logic | Custom scroll detection | `$effect.pre` + `tick()` pattern | Svelte 5 idiomatic approach |
| Log timestamp parsing | Regex on raw output | `LogsOptions.timestamps(true)` | Docker provides structured timestamps |

**Key insight:** Docker's log API already handles the complexity of multiplexing stdout/stderr with framing. Don't try to parse raw Docker API responses.

## Common Pitfalls

### Pitfall 1: Stream Doesn't End
**What goes wrong:** The log stream never completes, keeping the connection open indefinitely.
**Why it happens:** With `follow: true`, the stream stays open as long as the container is running.
**How to avoid:** Design for infinite streaming. Provide a "stop" button that cancels the stream. Consider using `tokio::select!` with a cancellation token.
**Warning signs:** Memory grows over time, app becomes unresponsive.

### Pitfall 2: Lost Logs on Container Stop
**What goes wrong:** When container stops, final logs may be lost.
**Why it happens:** Stream may close before all buffered logs are transmitted.
**How to avoid:** Handle stream errors gracefully, fetch final logs with `follow: false` if needed.
**Warning signs:** Missing log lines compared to `docker logs` output.

### Pitfall 3: UTF-8 Decoding Errors
**What goes wrong:** App crashes or shows garbled text for binary output.
**Why it happens:** Container output may contain non-UTF8 bytes (binary data, ANSI codes).
**How to avoid:** Always use `String::from_utf8_lossy()` which replaces invalid sequences with �.
**Warning signs:** Crashes on containers that output binary data.

### Pitfall 4: Memory Growth with Long-Running Streams
**What goes wrong:** App memory grows indefinitely while streaming logs.
**Why it happens:** Storing all logs in memory without limit.
**How to avoid:** Implement log rotation/truncation (e.g., keep last 10,000 lines). Consider virtual scrolling for large log volumes.
**Warning signs:** Performance degradation over time, eventual OOM.

### Pitfall 5: ANSI Escape Codes Not Rendered
**What goes wrong:** Color codes appear as raw text like `[32m` instead of colored output.
**Why it happens:** Docker logs preserve ANSI codes; frontend doesn't interpret them.
**How to avoid:** Either strip ANSI codes server-side, or use a library like `ansi-to-html` in frontend.
**Warning signs:** Log output contains `[0m`, `[1m`, `[32m` sequences.

## Code Examples

### Complete Backend Command (Verified Pattern)

```rust
// Source: Project patterns + bollard docs
use bollard::Docker;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::container::LogOutput;
use futures::StreamExt;
use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LogEvent {
    StdOut { message: String },
    StdErr { message: String },
    Error { message: String },
    Eof,
}

#[tauri::command]
pub async fn stream_container_logs(
    container_name: String,
    on_log: Channel<LogEvent>,
    tail: Option<u64>,
    timestamps: Option<bool>,
) -> Result<(), String> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("Failed to connect to Docker: {}", e))?;

    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(true)
        .tail(tail.map(|n| n.to_string()).unwrap_or_else(|| "100".to_string()))
        .timestamps(timestamps.unwrap_or(true))
        .build();

    let mut stream = docker.logs(&container_name, Some(options));

    while let Some(result) = stream.next().await {
        match result {
            Ok(LogOutput::StdOut { message }) => {
                let msg = String::from_utf8_lossy(&message).to_string();
                on_log.send(LogEvent::StdOut { message: msg })
                    .map_err(|e| format!("Channel send error: {}", e))?;
            }
            Ok(LogOutput::StdErr { message }) => {
                let msg = String::from_utf8_lossy(&message).to_string();
                on_log.send(LogEvent::StdErr { message: msg })
                    .map_err(|e| format!("Channel send error: {}", e))?;
            }
            Ok(_) => continue, // StdIn, Console - skip
            Err(e) => {
                on_log.send(LogEvent::Error {
                    message: format!("Stream error: {}", e),
                }).ok();
                break;
            }
        }
    }

    on_log.send(LogEvent::Eof).ok();
    Ok(())
}
```

### Stop Log Stream Command

```rust
// Pattern for stopping a running log stream
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

// In your state management
type CancelTokens = Arc<RwLock<HashMap<String, tokio_util::sync::CancellationToken>>>;

#[tauri::command]
pub async fn stop_log_stream(container_name: String) -> Result<(), String> {
    // Implementation would use CancellationToken to cancel the stream
    // This requires storing tokens per-container when stream starts
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `LogsOptions` struct builder | `LogsOptionsBuilder` default | bollard 0.17+ | More ergonomic API |
| `app.emit()` for all events | `ipc::Channel` for streams | Tauri v2 | Better performance for streaming |
| Svelte 4 `beforeUpdate` | Svelte 5 `$effect.pre` | Svelte 5 | Cleaner reactivity tracking |

**Deprecated/outdated:**
- `bollard::container::LogsOptions` struct pattern: Use `LogsOptionsBuilder` instead
- Svelte 4 lifecycle hooks: Use Svelte 5 runes in new code

## Open Questions

1. **Virtual scrolling for large log volumes**
   - What we know: Standard DOM rendering works for ~10k lines
   - What's unclear: Performance impact with 100k+ lines
   - Recommendation: Start with simple approach, add virtualization only if needed

2. **Log persistence/search**
   - What we know: Not in current requirements
   - What's unclear: Whether users will want to search/filter logs
   - Recommendation: Keep logs in memory only for this phase, defer search to future

3. **Multi-container log aggregation**
   - What we know: Not in current requirements
   - What's unclear: Whether users want to see logs from multiple containers
   - Recommendation: Single container per view for this phase

## Sources

### Primary (HIGH confidence)
- https://docs.rs/bollard/latest/bollard/container/enum.LogOutput.html - LogOutput enum documentation
- https://docs.rs/bollard/latest/bollard/struct.Docker.html#method.logs - logs() method documentation
- https://v2.tauri.app/develop/calling-frontend - IPC Channel documentation (Context7)
- https://github.com/sveltejs/svelte - Svelte 5 runes and $effect.pre patterns (Context7)

### Secondary (MEDIUM confidence)
- Project source files:
  - `src-tauri/src/docker/client.rs` - Existing event streaming pattern
  - `src-tauri/src/commands/images.rs` - Existing command pattern with AppHandle
  - `src/lib/components/PullProgress.svelte` - Existing event listening pattern

### Tertiary (LOW confidence)
- None required - all core patterns verified from primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use in project
- Architecture: HIGH - Patterns verified from Context7 and project code
- Pitfalls: MEDIUM - Based on documentation and common patterns, not direct testing

**Research date:** 2026-02-28
**Valid until:** 90 days (bollard and Tauri APIs are stable)
