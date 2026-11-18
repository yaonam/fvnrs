GameState defines the state of the game. This is what the rendering pipeline uses and gets saved to disk.

Schema defines the formatting of the game state used to communicate
between the various components of the game.

(e.g. script -> schema (GameState) -> ui)

It should be purely stateful and contain minimal logic.
