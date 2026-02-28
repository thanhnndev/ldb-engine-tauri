<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from 'svelte';
  import { SUPPORTED_IMAGES, type ImageTag, type DatabaseType, type CreateInstanceRequest } from '$lib/types';

  interface Props {
    onsubmit: (request: CreateInstanceRequest) => void;
    oncancel: () => void;
    loading?: boolean;
  }

  let { onsubmit, oncancel, loading = false }: Props = $props();

  let name = $state('');
  let databaseType = $state<DatabaseType>('postgres');
  let imageTag = $state('');
  let password = $state('');
  let port = $state<number | undefined>(undefined);

  let tags = $state<ImageTag[]>([]);
  let loadingTags = $state(false);
  let tagsError = $state<string | null>(null);
  let errors = $state<Record<string, string>>({});
  
  // Cache tags per database type to avoid re-fetching
  const tagsCache = new Map<DatabaseType, ImageTag[]>();
  
  // Debounce timer
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Group tags by category for better UX
  const groupedTags = $derived(() => {
    const groups: { recommended: ImageTag[]; alpine: ImageTag[]; slim: ImageTag[]; version: ImageTag[]; other: ImageTag[] } = {
      recommended: [],
      alpine: [],
      slim: [],
      version: [],
      other: []
    };
    
    for (const tag of tags) {
      const category = tag.category || 'other';
      if (category in groups) {
        groups[category as keyof typeof groups].push(tag);
      }
    }
    
    return groups;
  });

  async function loadTags() {
    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (!image) {
      tagsError = 'Unknown database type';
      loadingTags = false;
      return;
    }

    // Check cache first
    if (tagsCache.has(databaseType)) {
      tags = tagsCache.get(databaseType)!;
      // Set default tag to latest or first available
      const latestTag = tags.find(t => t.name === 'latest');
      imageTag = latestTag ? latestTag.name : (tags[0]?.name || '');
      return;
    }

    loadingTags = true;
    tagsError = null;
    tags = [];
    imageTag = '';
    
    try {
      // Add a timeout to prevent infinite loading
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Loading versions timed out')), 10000);
      });
      
      const fetchedTags = await Promise.race([
        invoke<ImageTag[]>("get_docker_tags", { image: image.hubName }),
        timeoutPromise
      ]);
      
      // Cache the tags for this database type
      tagsCache.set(databaseType, fetchedTags);
      tags = fetchedTags;
      
      // Set default tag to latest or first available
      const latestTag = tags.find(t => t.name === 'latest');
      imageTag = latestTag ? latestTag.name : (tags[0]?.name || '');
    } catch (e) {
      console.error("Failed to load tags:", e);
      tagsError = String(e);
      tags = [];
    } finally {
      loadingTags = false;
    }
  }

  function handleTypeChange() {
    // Auto-suggest port based on database type
    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (image) {
      port = image.default_port;
    }
    
    // Debounce tag loading to prevent rapid API calls
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    
    debounceTimer = setTimeout(() => {
      loadTags();
    }, 150);
  }

  function validate(): boolean {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      newErrors.name = 'Name can only contain letters, numbers, underscores, and hyphens';
    }

    if (!imageTag) {
      newErrors.imageTag = 'Please select a version';
    }

    if (!password) {
      newErrors.password = 'Password is required';
    } else if (password.length < 4) {
      newErrors.password = 'Password must be at least 4 characters';
    }

    if (port && (port < 1024 || port > 65535)) {
      newErrors.port = 'Port must be between 1024 and 65535';
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  function handleSubmit() {
    if (!validate()) return;

    const image = SUPPORTED_IMAGES.find(img => img.id === databaseType);
    if (!image) return;

    const request: CreateInstanceRequest = {
      name: name.trim(),
      database_type: databaseType,
      image: image.hubName,
      tag: imageTag,
      password,
      port: port || undefined
    };

    onsubmit(request);
  }

  // Load tags on mount only (handleTypeChange handles subsequent loads)
  onMount(() => {
    loadTags();
  });
</script>

<form class="instance-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <h3>Create New Instance</h3>

  <div class="form-group">
    <label for="name">Instance Name</label>
    <input 
      type="text" 
      id="name" 
      bind:value={name}
      placeholder="my-database"
      disabled={loading}
      class:error={errors.name}
    />
    {#if errors.name}
      <span class="error-message">{errors.name}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="databaseType">Database Type</label>
    <select 
      id="databaseType" 
      bind:value={databaseType}
      onchange={handleTypeChange}
      disabled={loading}
    >
      {#each SUPPORTED_IMAGES as image}
        <option value={image.id}>{image.name}</option>
      {/each}
    </select>
  </div>

  <div class="form-group">
    <label for="imageTag">Version</label>
    {#if loadingTags}
      <div class="loading-tags">
        <span class="spinner"></span>
        <span>Loading versions...</span>
      </div>
    {:else if tagsError}
      <div class="tags-error">
        <span class="error-text">{tagsError}</span>
        <button type="button" class="retry-btn" onclick={loadTags}>Retry</button>
      </div>
    {:else}
      <select 
        id="imageTag" 
        bind:value={imageTag}
        disabled={loading || tags.length === 0}
        class:error={errors.imageTag}
      >
        <option value="" disabled>Select version</option>
        
        {#if groupedTags().recommended.length > 0}
          <optgroup label="⭐ Recommended">
            {#each groupedTags().recommended as tag}
              <option value={tag.name}>
                {tag.name}
                {#if tag.name === 'latest'}
                  (stable)
                {:else if tag.name.includes('-alpine')}
                  (lightweight)
                {/if}
              </option>
            {/each}
          </optgroup>
        {/if}
        
        {#if groupedTags().alpine.length > 0}
          <optgroup label="🏔️ Alpine (Lightweight)">
            {#each groupedTags().alpine as tag}
              <option value={tag.name}>{tag.name}</option>
            {/each}
          </optgroup>
        {/if}
        
        {#if groupedTags().slim.length > 0}
          <optgroup label="📦 Slim">
            {#each groupedTags().slim as tag}
              <option value={tag.name}>{tag.name}</option>
            {/each}
          </optgroup>
        {/if}
        
        {#if groupedTags().version.length > 0}
          <optgroup label="📋 Version Tags">
            {#each groupedTags().version as tag}
              <option value={tag.name}>{tag.name}</option>
            {/each}
          </optgroup>
        {/if}
        
        {#if groupedTags().other.length > 0}
          <optgroup label="📁 Other">
            {#each groupedTags().other as tag}
              <option value={tag.name}>{tag.name}</option>
            {/each}
          </optgroup>
        {/if}
      </select>
      <span class="hint">{tags.length} versions available</span>
    {/if}
    {#if errors.imageTag}
      <span class="error-message">{errors.imageTag}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="password">
      {#if databaseType === 'redis'}
        Redis Password
      {:else if databaseType === 'mongo'}
        MongoDB Root Password
      {:else}
        Root Password
      {/if}
    </label>
    <input 
      type="password" 
      id="password" 
      bind:value={password}
      placeholder="Enter password"
      disabled={loading}
      class:error={errors.password}
    />
    {#if errors.password}
      <span class="error-message">{errors.password}</span>
    {/if}
  </div>

  <div class="form-group">
    <label for="port">Port (optional)</label>
    <input 
      type="number" 
      id="port" 
      bind:value={port}
      placeholder="Auto-detect"
      min="1024"
      max="65535"
      disabled={loading}
      class:error={errors.port}
    />
    <span class="hint">Leave empty for auto-detection</span>
    {#if errors.port}
      <span class="error-message">{errors.port}</span>
    {/if}
  </div>

  <div class="form-actions">
    <button type="button" class="btn cancel" onclick={oncancel} disabled={loading}>
      Cancel
    </button>
    <button type="submit" class="btn submit" disabled={loading}>
      {#if loading}
        <span class="spinner"></span>
        Creating...
      {:else}
        Create Instance
      {/if}
    </button>
  </div>
</form>

<style>
  .instance-form {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 1.5rem;
  }

  h3 {
    margin: 0 0 1.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
  }

  .form-group {
    margin-bottom: 1.25rem;
  }

  label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
    margin-bottom: 0.5rem;
  }

  input, select {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    font-size: 0.95rem;
    color: #1f2937;
    background: #f9fafb;
    transition: all 0.15s ease;
    box-sizing: border-box;
  }

  input:focus, select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
    background: white;
  }

  input.error, select.error {
    border-color: #ef4444;
  }

  input:disabled, select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  /* Style for version select dropdown */
  select#imageTag {
    cursor: pointer;
    max-height: 300px;
  }
  
  select#imageTag optgroup {
    font-weight: 600;
    color: #374151;
    background: #f3f4f6;
  }
  
  select#imageTag option {
    font-weight: 400;
    color: #4b5563;
    padding: 0.25rem 0.5rem;
  }

  .hint {
    display: block;
    font-size: 0.8rem;
    color: #6b7280;
    margin-top: 0.25rem;
  }

  .error-message {
    display: block;
    font-size: 0.8rem;
    color: #ef4444;
    margin-top: 0.25rem;
  }

  .loading-tags {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 0.875rem;
    color: #6b7280;
    font-size: 0.9rem;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
  }
  
  .loading-tags .spinner {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }

  .tags-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.625rem 0.875rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
  }

  .tags-error .error-text {
    color: #dc2626;
    font-size: 0.9rem;
    flex: 1;
  }

  .retry-btn {
    padding: 0.25rem 0.75rem;
    border: none;
    border-radius: 4px;
    background: #3b82f6;
    color: white;
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .retry-btn:hover {
    background: #2563eb;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid #e5e7eb;
    border-top-color: #3b82f6;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid #f3f4f6;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: none;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn.cancel {
    background: #f3f4f6;
    color: #374151;
  }

  .btn.cancel:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .btn.submit {
    background: #3b82f6;
    color: white;
  }

  .btn.submit:hover:not(:disabled) {
    background: #2563eb;
  }
</style>
