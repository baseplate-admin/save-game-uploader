<script lang="ts">
    import House from 'lucide-svelte/icons/house';
    import Cloud from 'lucide-svelte/icons/cloud';
    import RefreshCCW from 'lucide-svelte/icons/refresh-ccw';

    import * as Sidebar from '$lib/components/ui/sidebar/index.js';

    // Menu items.
    const items = {
        top: [
            {
                title: 'Home',
                url: '/home',
                icon: House,
            },

            {
                title: 'Google Drive',
                url: '/cloud',
                icon: Cloud,
            },
        ],
        bottom: [
            {
                title: 'Reload',
                url: '/loading',
                icon: RefreshCCW,
            },
        ],
    };
</script>

<Sidebar.Root>
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each items.top as item (item.title)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton>
                                {#snippet child({ props })}
                                    <a href={item.url} {...props}>
                                        <item.icon />
                                        <span>{item.title}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
    <Sidebar.Footer>
        <Sidebar.Menu>
            {#each items.bottom as item (item.title)}
                <Sidebar.MenuItem>
                    <Sidebar.MenuButton>
                        {#snippet child({ props })}
                            <a href={item.url} {...props}>
                                <item.icon />
                                <span>{item.title}</span>
                            </a>
                        {/snippet}
                    </Sidebar.MenuButton>
                </Sidebar.MenuItem>
            {/each}
        </Sidebar.Menu></Sidebar.Footer
    >
</Sidebar.Root>
