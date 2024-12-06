<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';

    onMount(async () => {
        await listen('main-loop-progress', (event) => {
            console.log(event);
        });
        invoke('find_games').then((res) => {
            console.log(res);
        });
    });
</script>

<main></main>

<style>
</style>
