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
        load_finished = $derived(
            total != null && current != null && games_store.state.length !== 0
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

        invoke('find_games').then((res) => {
            for (const item of res as GameJSON[]) {
                games_store.push(item);
            }
        });
    });
    $effect(() => {
        console.log(games_store.state.length);
    });
</script>

<div class="flex h-dvh w-full items-center justify-center flex-col gap-4">
    {#if load_finished}
        <div class="flex gap-2">
            <p>Loaded :</p>
            <b>{games_store.state.length}</b>
            <p>games</p>
        </div>
    {:else}
        <div>[{current}/{total}] {game_name}</div>
    {/if}
    <Progress value={current} max={total ?? 0} class="w-[60%]" />
</div>
