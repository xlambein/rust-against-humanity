use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitRequest {
	pub user_id: u64,
	pub cards: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
	Player,
	Czar,
}

fn default_n_answers() -> u8 { 1 }

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
	pub content: String,
	#[serde(default = "default_n_answers")]
	pub n_answers: u8,
}

impl Prompt {
	pub fn new(content: String, n_answers: u8) -> Self {
		Prompt { content, n_answers }
	}
}

impl fmt::Display for Prompt {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.n_answers == 1 {
			write!(f, "\"{}\"", self.content)
		} else {
			write!(f, "\"{}\" ({} answers)", self.content, self.n_answers)
		}
	}
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Answer {
	pub content: String,
}

impl Answer {
	pub fn new(content: String) -> Self {
		Answer { content }
	}
}

impl fmt::Display for Answer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.content)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginRejectedReason {
	UsernameIsTaken,
	GameIsFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
	Login(String),
	LoginAccepted,
	LoginRejected(LoginRejectedReason),
	// Register(String),
	// RegistrationRejected,
	JoinedLobby,
	Ready,
	NotReady,
	NewGame {
		hand: Vec<Answer>,
	},
	NewRound {
		role: Role,
		prompt: Prompt,
		hand: Vec<Answer>,
		// end_time: ,
	},
	// RoundTimeout,
	SubmitAnswer(Vec<Answer>),
	AnswerAccepted,
	AnswerRejected,
	ReadyToJudge(HashMap<usize, Vec<Answer>>),
	SubmitJudgement(usize),
	JudgementRejected,
	// JudgementTimeout,
	RoundEnded {
		winner: String,
		winning_answers: Vec<Answer>,
		scores: HashMap<String, u64>,
	},
	GameEnded,
	PlayerJoined {
		name: String,
	},
	PlayerLeft {
		name: String,
	},
}
