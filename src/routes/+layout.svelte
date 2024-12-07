<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import '../app.css';

    let { children } = $props();
    onMount(() => {
        goto('./loading');
    });
</script>

{@render children?.()}
