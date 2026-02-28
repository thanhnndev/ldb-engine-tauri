<script lang="ts">
  import type { Instance } from '$lib/types';
  import InstanceControls from './InstanceControls.svelte';
  import ConnectionString from './ConnectionString.svelte';
  import LogViewer from './LogViewer.svelte';

  interface Props {
    instance: Instance;
    onstart: () => void;
    onstop: () => void;
    onrestart: () => void;
    ondelete: (deleteVolume: boolean) => void;
    loading?: boolean;
  }

  let { instance, onstart, onstop, onrestart, ondelete, loading = false }: Props = $props();

  let showLogs = $state(false);

  // Format container name to match backend: "ldb-{name}"
  // Backend uses: format!("ldb-{}", request.name.replace(' ', "-").to_lowercase())
  const containerName = `ldb-${instance.name.replace(/\s+/g, '-').toLowerCase()}`;

  const statusColors: Record<string, string> = {
    running: '#22c55e',
    stopped: '#6b7280',
    error: '#ef4444',
    creating: '#f59e0b'
  };

  const statusLabels: Record<string, string> = {
    running: 'Running',
    stopped: 'Stopped',
    error: 'Error',
    creating: 'Creating'
  };

  const dbTypeIcons: Record<string, string> = {
    postgres: 'postgres',
    redis: 'redis',
    mysql: 'mysql',
    mongo: 'mongo'
  };
</script>

<div class="instance-card" class:running={instance.status === 'running'}>
  <div class="header">
    <div class="db-icon">
      {#if instance.database_type === 'postgres'}
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <ellipse cx="12" cy="5" rx="9" ry="3"/>
          <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
          <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
        </svg>
      {:else if instance.database_type === 'redis'}
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
          <path d="M8 12a2 2 0 1 0 4 0 2 2 0 0 0-4 0z"/>
        </svg>
      {:else if instance.database_type === 'mysql'}
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
          <path d="M2 17l10 5 10-5"/>
          <path d="M2 12l10 5 10-5"/>
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 17l6-6-6-6"/>
          <path d="M12 17h8"/>
        </svg>
      {/if}
    </div>
    <div class="info">
      <h3>{instance.name}</h3>
      <p class="image-tag">{instance.image}:{instance.tag}</p>
    </div>
    <div class="status-badge" style="background-color: {statusColors[instance.status]}20; color: {statusColors[instance.status]}">
      {statusLabels[instance.status]}
    </div>
  </div>
  
  <div class="details">
    <div class="detail-item">
      <span class="label">Type:</span>
      <span class="value">{instance.database_type}</span>
    </div>
    <div class="detail-item">
      <span class="label">Port:</span>
      <span class="value">{instance.port}</span>
    </div>
    {#if instance.volume_path}
      <div class="detail-item">
        <span class="label">Volume:</span>
        <span class="value path">{instance.volume_path}</span>
      </div>
    {/if}
  </div>

  {#if instance.status === 'running'}
    <ConnectionString 
      instanceId={instance.id}
      instanceName={instance.name}
      databaseType={instance.database_type}
      port={instance.port}
      status={instance.status}
    />
  {/if}

  {#if instance.status === 'running'}
    <div class="log-actions">
      <button class="view-logs-btn" onclick={() => showLogs = true}>
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
        View Logs
      </button>
    </div>
  {/if}

  <InstanceControls 
    status={instance.status}
    {onstart}
    {onstop}
    {onrestart}
    {ondelete}
    {loading}
  />
</div>

{#if showLogs}
  <div class="log-modal" onclick={() => showLogs = false} role="dialog" aria-modal="true" aria-label="Container logs">
    <div class="log-modal-content" onclick={(e) => e.stopPropagation()}>
      <LogViewer 
        {containerName}
        onclose={() => showLogs = false}
      />
    </div>
  </div>
{/if}

<style>
  .instance-card {
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 1.25rem;
    background: white;
    transition: all 0.2s ease;
  }

  .instance-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  .instance-card.running {
    border-color: #22c55e;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .db-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #eff6ff;
    border-radius: 10px;
    color: #3b82f6;
  }

  .info {
    flex: 1;
  }

  .info h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: #1f2937;
  }

  .image-tag {
    margin: 0.25rem 0 0;
    font-size: 0.85rem;
    color: #6b7280;
    font-family: monospace;
  }

  .status-badge {
    padding: 0.375rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .details {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 0.75rem 0;
    border-top: 1px solid #f3f4f6;
    border-bottom: 1px solid #f3f4f6;
    margin-bottom: 1rem;
  }

  .detail-item {
    display: flex;
    gap: 0.5rem;
    font-size: 0.9rem;
  }

  .label {
    color: #6b7280;
  }

  .value {
    color: #374151;
    font-weight: 500;
  }

  .value.path {
    font-family: monospace;
    font-size: 0.8rem;
    color: #6b7280;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .log-actions {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 0.75rem;
  }

  .view-logs-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    background: white;
    font-size: 0.85rem;
    font-weight: 500;
    color: #374151;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .view-logs-btn:hover {
    background: #f9fafb;
    border-color: #9ca3af;
  }

  .view-logs-btn svg {
    flex-shrink: 0;
  }

  .log-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .log-modal-content {
    max-width: 90vw;
    max-height: 90vh;
  }
</style>
