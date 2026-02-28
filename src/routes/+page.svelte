<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SUPPORTED_IMAGES, type ImageTag, type PullProgress as PullProgressType } from "$lib/types";
  import ImageCard from "$lib/components/ImageCard.svelte";
  import TagList from "$lib/components/TagList.svelte";
  import PullProgress from "$lib/components/PullProgress.svelte";

  let selectedImage = $state<string | null>(null);
  let tags = $state<ImageTag[]>([]);
  let selectedTag = $state<string | null>(null);
  let loadingTags = $state(false);
  let pulling = $state(false);
  let pullComplete = $state(false);

  async function loadTags(imageHubName: string) {
    loadingTags = true;
    tags = [];
    selectedTag = null;
    
    try {
      tags = await invoke<ImageTag[]>("get_docker_tags", { image: imageHubName });
    } catch (e) {
      console.error("Failed to load tags:", e);
      tags = [];
    } finally {
      loadingTags = false;
    }
  }

  function selectImage(id: string) {
    const image = SUPPORTED_IMAGES.find(img => img.id === id);
    if (image) {
      selectedImage = id;
      loadTags(image.hubName);
      pullComplete = false;
    }
  }

  function selectTag(tag: string) {
    selectedTag = tag;
  }

  async function pullImage() {
    const image = SUPPORTED_IMAGES.find(img => img.id === selectedImage);
    if (!image || !selectedTag) return;
    
    pulling = true;
    try {
      await invoke("pull_docker_image", { image: `${image.hubName}:${selectedTag}` });
    } catch (e) {
      console.error("Pull failed:", e);
    } finally {
      pulling = false;
    }
  }

  function onPullComplete() {
    pullComplete = true;
  }

  function cancelPull() {
    pulling = false;
  }

  function reset() {
    selectedImage = null;
    tags = [];
    selectedTag = null;
    pulling = false;
    pullComplete = false;
  }

  let fullImageName = $derived(
    selectedImage && selectedTag 
      ? `${SUPPORTED_IMAGES.find(i => i.id === selectedImage)?.hubName}:${selectedTag}`
      : ''
  );
</script>

<main class="container">
  <h1>LDB-Engine</h1>
  <p class="subtitle">Local Database Manager</p>

  {#if !selectedImage}
    <section class="select-image">
      <h2>Select a Database</h2>
      <div class="image-grid">
        {#each SUPPORTED_IMAGES as image}
          <ImageCard 
            {image} 
            selected={selectedImage === image.id}
            loading={false}
            ontoggle={() => selectImage(image.id)}
          />
        {/each}
      </div>
    </section>
  {:else}
    <section class="configure">
      <button class="back-btn" onclick={reset}>
        ← Back to selection
      </button>
      
      <div class="selected-info">
        <h2>{SUPPORTED_IMAGES.find(i => i.id === selectedImage)?.name}</h2>
        <p class="image-name">{fullImageName}</p>
      </div>

      <TagList 
        {tags} 
        {selectedTag} 
        loading={loadingTags}
        onselect={selectTag}
      />

      {#if pulling}
        <PullProgress 
          image={fullImageName}
          oncomplete={onPullComplete}
          oncancel={cancelPull}
        />
      {:else if pullComplete}
        <div class="success-message">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          <span>Image pulled successfully!</span>
          <button class="pull-again-btn" onclick={() => { pullComplete = false; }}>
            Pull Again
          </button>
        </div>
      {:else if selectedTag}
        <button class="pull-btn" onclick={pullImage}>
          Pull Image
        </button>
      {/if}
    </section>
  {/if}
</main>

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  h1 {
    text-align: center;
    font-size: 2.5rem;
    font-weight: 700;
    color: #1f2937;
    margin: 0;
  }
  
  .subtitle {
    text-align: center;
    color: #6b7280;
    margin: 0.5rem 0 2rem;
  }
  
  h2 {
    font-size: 1.25rem;
    font-weight: 600;
    color: #374151;
    margin: 0 0 1rem;
  }
  
  .image-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 1rem;
  }
  
  .select-image {
    margin-top: 2rem;
  }
  
  .configure {
    margin-top: 1rem;
  }
  
  .back-btn {
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    color: #3b82f6;
    font-size: 0.9rem;
    cursor: pointer;
    margin-bottom: 1rem;
  }
  
  .back-btn:hover {
    text-decoration: underline;
  }
  
  .selected-info {
    margin-bottom: 1.5rem;
  }
  
  .selected-info h2 {
    margin-bottom: 0.25rem;
  }
  
  .image-name {
    color: #6b7280;
    font-family: monospace;
    font-size: 0.9rem;
    margin: 0;
  }
  
  .pull-btn {
    width: 100%;
    margin-top: 1.5rem;
    padding: 0.875rem;
    border: none;
    border-radius: 8px;
    background: #3b82f6;
    color: white;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s ease;
  }
  
  .pull-btn:hover {
    background: #2563eb;
  }
  
  .success-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding: 1rem;
    background: #ecfdf5;
    border-radius: 8px;
    color: #059669;
  }
  
  .pull-again-btn {
    margin-left: auto;
    padding: 0.375rem 0.75rem;
    border: none;
    border-radius: 6px;
    background: #059669;
    color: white;
    font-size: 0.85rem;
    cursor: pointer;
  }
</style>
