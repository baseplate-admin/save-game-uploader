import { SvelteSet } from 'svelte/reactivity';

export type GameJSON = {
    name: string;
    parent: string;
    directory: string;
    globs: string[];
    image: string;
};

let games_store = $state(new SvelteSet<GameJSON>());

export function createGamesStore() {
    return {
        get state() {
            return games_store;
        },

        reset() {
            games_store = new SvelteSet<GameJSON>();
        },
    };
}
