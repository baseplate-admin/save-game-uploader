export type GameJSON = {
    name: string;
    parent: string;
    directory: string;
    globs: string[];
    image: string;
};

let games_store = $state<GameJSON[]>(new Array<GameJSON>());

export function createGamesStore() {
    return {
        get state() {
            return games_store;
        },
        push(payload: GameJSON) {
            games_store!.push(payload);
        },
        reset() {
            games_store = new Array<GameJSON>();
        },
    };
}
