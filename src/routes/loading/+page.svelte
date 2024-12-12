<script lang="ts">
    import { onMount } from 'svelte';
    import { Progress } from '$lib/components/ui/progress/index.js';
    import { listen } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/core';
    import { createGamesStore, type GameJSON } from '$lib/stores/games.svelte';
    import { goto } from '$app/navigation';

    const games_store = createGamesStore();

    interface EventPayload {
        name: string;
        total: number;
        current: number;
    }

    let total: number | null = $state(null),
        current: number | null = $state(null),
        game_name: string | null = $state(null),
        load_finished: boolean | null = $derived(
            total === current && games_store.state.size > 0
        );

    $effect(() => {
        if (load_finished) {
            console.log('Done');
        }
    });

    onMount(async () => {
        games_store.reset();

        await listen('main-loop-progress', (event) => {
            const payload = event.payload as EventPayload;

            total = payload.total;
            current = payload.current;
            game_name = payload.name;
        });

        await invoke('find_games').then((res) => {
            for (const item of res as GameJSON[]) {
                games_store.state.add(item);
            }
        });
    });
</script>

<div class="flex h-dvh w-full items-center justify-center flex-col gap-4">
    {#if load_finished}
        <div class="flex gap-2">
            <p>Loaded :</p>
            <b>{games_store.state.size}</b>
            <p>games</p>
        </div>
    {:else}
        <div>[{current}/{total}] {game_name}</div>
    {/if}
    {#if total && current}
        <Progress value={current} max={total} class="w-[60%]" />
    {/if}
</div>
