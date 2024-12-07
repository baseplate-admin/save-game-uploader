<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import * as Sidebar from '$lib/components/ui/sidebar/index.js';
    import AppSidebar from '$lib/components/app-sidebar.svelte';
    import { onMount } from 'svelte';

    let { children } = $props();
    onMount(() => {});
</script>

<Sidebar.Provider>
    <AppSidebar />
    <main>
        {@render children?.()}
    </main>
</Sidebar.Provider>
