<script lang="ts">
  import InstanceList from "$lib/components/InstanceList.svelte";
</script>

<main class="container">
  <h1>LDB-Engine</h1>
  <p class="subtitle">Local Database Manager</p>

  <InstanceList />
</main>

<style>
  .container {
    max-width: 900px;
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
</style>
