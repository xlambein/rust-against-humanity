# Rust Against Humanity

A web version of Cards Against Humanity, made in Rust with [Yew](https://yew.rs) (frontend) and [warp](https://github.com/seanmonstar/warp) (backend).

## How to Run

From the root of the repository, start the backend with:

```bash
cargo run -p back
```

and the frontend with:

```bash
npm start --prefix front
```

You can join a game by opening the <http://0.0.0.0:7777> URL.  This should also work on your LAN, if the port is open.

## Directory Structure

- `/schema/` is the common crate between the front- and backend
- `/back/` is the backend
- `/front/` is the frontend
- `/assets/` contains the data for the "prompt" and "answer" cards

## To-Do

- [x] Load cards from file/database
- [x] Handle client (dis)connection
- [x] Client name selection
- [x] Limit number of players in game
- [x] Automatic "_" expansion
- [x] Better judgement screen
- [x] Improve the design of everything
- [x] Cycle Czars
- [x] Shuffle prompts at start
- [x] If player joins during judgement, they shouldn't be able to play
- [x] Minimum number of players
- [x] Sort scores

- [ ] Notifications for events (player (dis)connection)
- [ ] Who's online?
- [ ] Display "You won" instead of "{username} won" :P
- [ ] Log-in with HTTP, and then only connect via websocket
- [ ] Remember scores
- [ ] Serve frontend with the backend

- [ ] End of game
- [ ] Lobby
- [ ] Shuffle jugement cards before displaying

1. Client opens web app.  They select a user name.
2. They click "join" and are added to the current game.
3. They play the game.
4. If they are disconnected:
   - If they're a player, they simply leave the game.  Their score is kept.
   - If they're the Czar, the round ends.  Players get their cards back.  New round starts
5. If they re-join with the same username, they get their score back.
