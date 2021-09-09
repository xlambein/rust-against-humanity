use anyhow::{Result, bail};
use futures::{FutureExt, StreamExt};
use warp::Filter;
use warp::ws::{Message, WebSocket};
use tokio::sync::{mpsc, RwLock};
use std::collections::{HashMap, hash_map};
use std::sync::{
	Arc,
	atomic::{AtomicUsize, Ordering},
};
use rand::seq::IteratorRandom;
use std::convert::Infallible;

use schema::{Message as WsMsg, Role, Prompt, Answer, LoginRejectedReason};

mod util;
mod deck;

use util::expand_underscores;
use deck::Deck;


#[derive(Default)]
struct Game {
	prompts: Deck<Prompt>,
	answers: Deck<Answer>,
	round: Option<Round>,
	clients: HashMap<usize, mpsc::UnboundedSender<schema::Message>>,
	players: HashMap<usize, Player>,
}

static N_CARDS_IN_HAND: usize = 4;
static MIN_N_PLAYERS: usize = 3;
static MAX_N_PLAYERS: usize = 3;
static N_UNDERSCORES: usize = 5;

impl Game {
	fn distribute_cards(&mut self) {
		for player in &mut self.players.values_mut() {
			if player.hand.len() < N_CARDS_IN_HAND {
				player.hand.extend(self.answers.draw(N_CARDS_IN_HAND - player.hand.len()));
			}
		}
	}

	fn new_round(&mut self) -> Result<()> {
		if self.players.len() == 0 {
			bail!("There are no players!");
		}

		let mut next_czar = 0;
		
		// Discard current round
		if let Some(Round{ prompt, answers, czar, .. }) = self.round.take() {
			next_czar = czar+1;
			self.prompts.discard(&[prompt]);
			for cards in answers.values() {
				self.answers.discard(cards);
			}
		}

		// Find next czar
		let mut player_ids = self.players.keys().collect::<Vec<_>>();
		player_ids.sort_unstable();
		if let Err(idx) = player_ids.binary_search(&&next_czar) {
			// There's no player with ID next_czar
			if idx == player_ids.len() {
				// There isn't a greater key
				next_czar = *player_ids[0];
			} else {
				// There is a key greater than next_czar
				next_czar = *player_ids[idx];
			}
		}

		// Create new round
		println!("Players to choose from: {:?}", self.players.keys().map(|u| u.to_string()).collect::<Vec<_>>().join(", "));
		let round = Round {
			prompt: self.prompts.draw_once(),
			// TODO cycle Czars
			czar: next_czar,
			answers: Default::default(),
			state: RoundState::Answering,
		};

		println!("Next czar is Player #{}", round.czar);

		// Distribute cards and notify players
		self.distribute_cards();
		for (id, player) in &mut self.players {
			let role = if *id == round.czar { Role::Czar } else { Role::Player };
			self.clients[id].send(WsMsg::NewRound {
				role,
				prompt: round.prompt.clone(),
				hand: player.hand.clone(),
			})?;
		}

		// Set new round
		self.round = Some(round);

		Ok(())
	}

	fn broadcast_to_players(&mut self, msg: &WsMsg) -> Result<()> {
		for id in self.players.keys() {
			self.clients[id].send(msg.clone())?;
		}
		Ok(())
	}
}

#[derive(PartialEq)]
enum RoundState {
	Answering,
	Judging,
}

struct Round {
	prompt: Prompt,
	czar: usize,
	answers: HashMap<usize, Vec<Answer>>,
	state: RoundState,
}

struct Player {
	name: String,
	hand: Vec<Answer>,
	score: u64,
}

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

async fn process_message(
	game: &Arc<RwLock<Game>>,
	user_id: usize,
	msg: WsMsg,
	tx: &mpsc::UnboundedSender<WsMsg>
) -> Result<()> {
	match msg {
		WsMsg::Login(username) => {
			if game.read().await.players.len() >= MAX_N_PLAYERS {
				tx.send(WsMsg::LoginRejected(LoginRejectedReason::GameIsFull))?;
				return Ok(())
			}
			if game.read().await.players.values().any(|player| player.name == username) {
				tx.send(WsMsg::LoginRejected(LoginRejectedReason::UsernameIsTaken))?;
				return Ok(())
			}
			tx.send(WsMsg::LoginAccepted)?;

			let hand = game.write().await.answers.draw(N_CARDS_IN_HAND);
		
			let player = Player {
				name: username.clone(),
				hand: hand.clone(),
				score: 0,
			};
		
			game.write().await.players.insert(user_id, player);

			// Notify other players
			game.write().await.broadcast_to_players(&WsMsg::PlayerJoined { name: username })?;

			// Only start new round if there are enough players
			if game.read().await.players.len() >= MIN_N_PLAYERS {
				let game = &mut game.write().await;
		
				let round = if let Some(round) = &game.round {
					round
				} else {
					// TODO lobby
					println!("Starting new round");
					game.new_round()?;
					game.round.as_ref().unwrap()
				};

				// If in judgement, don't send NewRound
				if round.state == RoundState::Answering {
					let role = if round.czar == user_id { Role::Czar } else { Role::Player };
					tx.send(WsMsg::NewRound {
						role,
						prompt: round.prompt.clone(),
						hand: hand,
					})?;
				}
			}

			Ok(())
		},

		// WsMsg::Register(name) => todo!(),
		// WsMsg::Ready => todo!(),
		// WsMsg::NotReady => todo!(),

		WsMsg::SubmitAnswer(answers) => {
			if let Game {
				clients,
				players,
				round: Some(round),
				..
			} = &mut *game.write().await {
				if round.state != RoundState::Answering {
					eprintln!("invalid query SubmitAnswer: round is in judgement phase");
					return Ok(())
				}

				if round.czar == user_id {
					eprintln!("invalid query SubmitAnswer: player is Czar");
					return Ok(())
				}

				match round.answers.entry(user_id) {
					hash_map::Entry::Occupied(_) => {
						eprintln!("invalid query SubmitAnswer: player already submitted answer")
					},
					hash_map::Entry::Vacant(entry) => {
						let hand = &mut players.get_mut(&user_id).unwrap().hand;
						if !answers.iter().all(|x| hand.contains(x)) {
							eprintln!("invalid query SubmitAnswer: cards are not in player's deck");
							return Ok(())
						}
						println!("SubmitAnswer({})", answers.iter().map(Answer::to_string).collect::<Vec<_>>().join(", "));
						// Remove cards from player's hand
						hand.retain(|x| !answers.contains(x));
						// Insert cards into submitted answers
						entry.insert(answers);
						tx.send(WsMsg::AnswerAccepted)?;
					},
				}

				// Check whether all players have answered
				if round.answers.len() == players.len() - 1 {
					round.state = RoundState::Judging;
					// If so, notify them that JUDGEMENT HAS BEGUN
					// TODO maybe obfuscate the player IDs before sending
					for id in players.keys() {
						clients[id].send(WsMsg::ReadyToJudge(round.answers.clone()))?;
					}
				}
			} else {
				eprintln!("invalid query SubmitAnswer: there is no ongoing round");
			}
			// TODO send AnswerAccepted/Rejected messages
			Ok(())
		},

		WsMsg::SubmitJudgement(answer_id) => {
			let mut new_round = false;

			if let Game {
				clients,
				players,
				round: Some(round),
				..
			} = &mut *game.write().await {
				if round.state != RoundState::Judging {
					eprintln!("invalid query SubmitAnswer: round is in judgement phase");
					return Ok(())
				}

				if round.czar != user_id {
					eprintln!("invalid query SubmitJudgement: player isn't Czar");
					return Ok(())
				}

				match round.answers.get(&answer_id) {
					None => {
						eprintln!("invalid query SubmitJudgement: user ID does not exist");
					},
					Some(winning_answers) => {
						let winner = {
							// Increment winner's scores
							let winner = players.get_mut(&answer_id).unwrap();
							winner.score += 1;
							// Get winner's name
							winner.name.clone()
						};
						let scores = players.values().map(|player| (player.name.clone(), player.score)).collect();
						let msg = WsMsg::RoundEnded {
							winner,
							winning_answers: winning_answers.clone(),
							scores,
						};

						// Notify end of round, provide winner and scores
						for id in players.keys() {
							clients[id].send(msg.clone())?;
						}

						new_round = true;
					}
				}
			} else {
				eprintln!("invalid query SubmitAnswer: there is no ongoing round");
			}

			if new_round {
				game.write().await.new_round()?;
			}

			// TODO send JudgementAccepted/Rejected messages
			Ok(())
		},

		_ => unreachable!(),
	}
}

async fn user_connected(game: Arc<RwLock<Game>>, socket: WebSocket) {
	let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

	println!("User connected: #{}", my_id);
	
	let (tx, mut rx) = socket.split();

	// Manage outgoing messages to this user
	let tx = {
		let (tx2, rx) = mpsc::unbounded_channel();
		tokio::task::spawn(rx.map(|msg| {
			Ok(Message::text(serde_json::to_string(&msg).unwrap()))
		}).forward(tx).map(move |result| {
			if let Err(e) = result {
				eprintln!("websocket send error: {}", e);
			}
		}));
		tx2
	};

	game.write().await.clients.insert(my_id, tx.clone());

	// Manage incoming messages from this user
	while let Some(result) = rx.next().await {
		let msg = match result {
			Ok(msg) => msg,
			Err(e) => {
				eprintln!("websocket error with user {}: {}", my_id, e);
				break;
			}
		};
		
		if let Ok(text) = msg.to_str() {
			if let Ok(response) = serde_json::from_str::<WsMsg>(text) {
				if let Err(_) = process_message(&game, my_id, response, &tx).await {
					eprintln!("Error while processing message from player #{}", my_id);
					break;
				}
			} else {
				eprintln!("cannot read message");
			}
		}
	}

	println!("Client #{} disconnected", my_id);
	user_disconnected(game, my_id).await;
}

async fn user_disconnected(game: Arc<RwLock<Game>>, user_id: usize) {
	let game = &mut *game.write().await;
	game.clients.remove(&user_id);

	if let Some(player) = game.players.remove(&user_id) {
		// Discard player's answers
		game.answers.discard(&player.hand);

		// Discard player's submitted answers, if any
		let mut user_is_czar = false;
		if let Game {
			answers,
			round: Some(Round { answers: submitted_answers, czar, .. }),
			..
		} = game {
			if let Some(cards) = submitted_answers.remove(&user_id) {
				answers.discard(&cards);
			}
			user_is_czar = *czar == user_id;
		}

		// If player is Czar, return submitted answers to owners and restart round
		if user_is_czar {
			let mut round = game.round.take().unwrap();
			game.prompts.discard(&[round.prompt]);
			for (id, player) in game.players.iter_mut() {
				player.hand.extend(round.answers.remove(id).into_iter().flatten());
			}
			if game.players.len() > 0 {
				game.new_round().expect("Couldn't start new round");
			}
		}

		// Notify other players
		game.broadcast_to_players(&WsMsg::PlayerLeft { name: player.name.clone() });
	}

	// If not enough players, cancel round
	if game.players.len() < MIN_N_PLAYERS {
		game.round = None;
		game.answers.reset();
		game.prompts.reset();

		for id in game.players.keys() {
			game.clients[id].send(WsMsg::GameEnded);
		}

		// Clear player hands, to avoid double-discard
		for player in game.players.values_mut() {
			player.hand.clear();
		}
	}
}

use ron;
use std::fs::File;
use serde::de::DeserializeOwned;

fn load_deck<Card: DeserializeOwned>(filename: &str) -> Result<Vec<Card>, Box<dyn std::error::Error>> {
	let file = File::open(filename)?;
	Ok(ron::de::from_reader(file)?)
}

fn load_prompts(filename: &str) -> Result<impl Iterator<Item=Prompt>, Box<dyn std::error::Error>> {
	Ok(load_deck::<Prompt>(filename)?
		.into_iter()
		.map(|prompt| {
			Prompt::new(
				expand_underscores(&prompt.content, N_UNDERSCORES),
				prompt.n_answers
			)
		}))
}

// async fn login(username: String, game: Arc<RwLock<Game>>) -> Result<impl warp::Reply, Infallible> {
// 	Ok(warp::reply::json(&WsMsg::LoginAccepted));
// 	Ok(warp::reply::json(&WsMsg::LoginRejected(LoginRejectedReason::GameIsFull)))
// }

#[tokio::main]
async fn main() {
	let mut game_state = Game::default();
	game_state.prompts.extend(load_prompts("assets/prompts.ron").unwrap());
	game_state.answers.extend(load_deck("assets/answers.ron").unwrap());

	let game_state = Arc::new(RwLock::new(game_state));
	let game_state = warp::any().map(move || game_state.clone());

	// warp::path!("login")
	// 	.and(warp::post())
	// 	.and(warp::body::json())
	// 	.and(game_state.clone())
	// 	.and_then(login);

	let login = warp::path::end()
		.map(|| {
			"Hello World!"
		});
	let game = warp::path::end()
		.and(warp::ws())
		.and(game_state)
		.map(|ws: warp::ws::Ws, game| {
			ws.on_upgrade(move |socket| user_connected(game, socket))
		});

	// Match any request and return hello world!
	let routes = game.or(login);

	warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
