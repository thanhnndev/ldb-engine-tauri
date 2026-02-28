<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Channel } from '@tauri-apps/api/core';
  import { tick } from 'svelte';

  // Log event types matching Rust backend
  // Rust serde serialization: {"type": "StdOut", "data": {"message": "log text"}}
  interface LogEventData {
    message: string;
  }

  interface LogEvent {
    type: 'StdOut' | 'StdErr' | 'Error' | 'Eof';
    data?: LogEventData;
  }

  interface LogLine {
    type: 'stdout' | 'stderr' | 'error';
    message: string;
  }

  interface Props {
    containerName: string;
    tail?: number;
    onclose: () => void;
  }

  let { containerName, tail = 100, onclose }: Props = $props();

  let logs = $state<LogLine[]>([]);
  let isStreaming = $state(false);
  let error = $state<string | null>(null);
  let logContainer: HTMLDivElement | undefined = $state();

  const MAX_LOG_LINES = 5000;

  // Auto-scroll effect - scroll to bottom when new logs arrive if user is near bottom
  $effect.pre(() => {
    logs; // reactivity trigger
    tick().then(() => {
      if (logContainer) {
        const nearBottom = 
          logContainer.offsetHeight + logContainer.scrollTop > 
          logContainer.scrollHeight - 50;
        if (nearBottom) {
          logContainer.scrollTop = logContainer.scrollHeight;
        }
      }
    });
  });

  // Start log streaming when component mounts
  $effect(() => {
    startLogStream();
    // Cleanup on unmount - the channel will be cleaned up by the backend
    // when it detects the channel is dropped
  });

  async function startLogStream() {
    try {
      isStreaming = true;
      error = null;
      logs = [];

      const onLog = new Channel<LogEvent>();
      
      onLog.onmessage = (event: LogEvent) => {
        switch (event.type) {
          case 'StdOut':
            addLog('stdout', event.data?.message || '');
            break;
          case 'StdErr':
            addLog('stderr', event.data?.message || '');
            break;
          case 'Error':
            error = event.data?.message || 'Unknown error';
            isStreaming = false;
            break;
          case 'Eof':
            isStreaming = false;
            break;
        }
      };

      await invoke('stream_container_logs', {
        containerName,
        onLog,
        tail
      });
    } catch (err) {
      error = String(err);
      isStreaming = false;
    }
  }

  function addLog(type: LogLine['type'], message: string) {
    logs.push({ type, message });
    
    // Enforce line limit to prevent memory growth
    if (logs.length > MAX_LOG_LINES) {
      logs = logs.slice(-MAX_LOG_LINES);
    }
  }

  function handleClose() {
    onclose();
  }
</script>

<div class="log-viewer">
  <div class="log-header">
    <div class="header-left">
      <h3>Logs: {containerName}</h3>
      {#if isStreaming}
        <span class="live-indicator">
          <span class="live-dot"></span>
          Live
        </span>
      {/if}
    </div>
    <button class="close-btn" onclick={handleClose} aria-label="Close log viewer">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>{error}</span>
    </div>
  {/if}

  <div class="log-content" bind:this={logContainer}>
    {#if logs.length === 0 && !error}
      <div class="empty-state">
        {#if isStreaming}
          <span class="spinner"></span>
          <span>Waiting for logs...</span>
        {:else}
          <span>No logs available</span>
        {/if}
      </div>
    {:else}
      {#each logs as log (log.message + log.type)}
        <div class="log-line {log.type}">
          <span class="log-message">{log.message}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    border-radius: 12px;
    overflow: hidden;
    width: 100%;
    max-width: 800px;
    max-height: 500px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  }

  .log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: #2d2d2d;
    border-bottom: 1px solid #404040;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .log-header h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: #d4d4d4;
    font-family: monospace;
  }

  .live-indicator {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #22c55e;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .live-dot {
    width: 8px;
    height: 8px;
    background: #22c55e;
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #9ca3af;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: #404040;
    color: #f3f4f6;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: #7f1d1d;
    color: #fca5a5;
    font-size: 0.85rem;
  }

  .log-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem 1rem;
    max-height: 400px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.8rem;
    line-height: 1.5;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    height: 100%;
    min-height: 100px;
    color: #6b7280;
    font-size: 0.85rem;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid #404040;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .log-line {
    padding: 0.125rem 0;
    word-break: break-all;
    white-space: pre-wrap;
  }

  .log-line.stdout .log-message {
    color: #d4d4d4;
  }

  .log-line.stderr .log-message {
    color: #ef4444;
  }

  .log-line.error .log-message {
    color: #fca5a5;
  }

  .log-message {
    display: block;
  }

  /* Scrollbar styling */
  .log-content::-webkit-scrollbar {
    width: 8px;
  }

  .log-content::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .log-content::-webkit-scrollbar-thumb {
    background: #404040;
    border-radius: 4px;
  }

  .log-content::-webkit-scrollbar-thumb:hover {
    background: #525252;
  }
</style>
