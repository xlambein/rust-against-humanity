use std::iter::Extend;
use rand::seq::SliceRandom;
use std::mem;

pub struct Deck<Card> {
	cards: Vec<Card>,
	remaining: Vec<usize>,
	discarded: Vec<usize>,
}

impl<Card> Default for Deck<Card> {
	fn default() -> Deck<Card> {
		Deck {
			cards: vec![],
			remaining: vec![],
			discarded: vec![],
		}
	}
}

impl<Card: Clone + Eq> Extend<Card> for Deck<Card> {
	fn extend<T: IntoIterator<Item=Card>>(&mut self, iter: T) {
		for elem in iter {
			self.add(elem);
		}
	}
}

fn reshuffle(remaining: &mut Vec<usize>, discarded: &mut Vec<usize>) {
	*remaining = mem::take(discarded);
	remaining.shuffle(&mut rand::thread_rng());
}

impl<Card: Clone + Eq> Deck<Card> {
	pub fn add(&mut self, card: Card) {
		self.cards.push(card);
		self.discarded.push(self.cards.len()-1);
	}

	pub fn draw(&mut self, n_cards: usize) -> Vec<Card> {
		// TODO make this method faster maybe?
		let mut cards = Vec::with_capacity(n_cards);

		for _ in 0..n_cards {
			let card = match self.remaining.pop() {
				Some(card) => card,
				None => {
					reshuffle(&mut self.remaining, &mut self.discarded);
					self.remaining.pop().expect("No cards are left to draw!")
				}
			};
			cards.push(self.cards[card].clone());
		}
		
		cards
	}

	pub fn draw_once(&mut self) -> Card {
		self.cards[match self.remaining.pop() {
			Some(card) => card,
			None => {
				reshuffle(&mut self.remaining, &mut self.discarded);
				self.remaining.pop().expect("Deck is empty!")
			}
		}].clone()
	}

	pub fn discard(&mut self, cards: &[Card]) {
		for card in cards {
			let i = self.cards.iter().position(|c| c == card).expect("Tried to discard a card not in deck");
			if self.remaining.contains(&i) {
				panic!("Tried to discard a card not drawn");
			}
			if self.discarded.contains(&i) {
				panic!("Tried to discard a card twice");
			}
			self.discarded.push(i);
		}
	}

	pub fn reset(&mut self) {
		self.remaining.clear();
		self.discarded = (0..self.cards.len()).collect();
		// self.discarded.extend(0..self.cards.len());
	}

	pub fn cards(&self) -> &Vec<Card> {
		&self.cards
	}
}

#[cfg(test)]
mod tests {
	// Note this useful idiom: importing names from outer (for mod tests) scope.
	use std::collections::HashSet;

	use super::*;

	#[test]
	fn test_deck_draw_no_repeat() {
		for _ in 0..100 {
			let mut deck = Deck::<i32>::default();
			for i in 0..12 {
				deck.add(i);
			}

			let mut set = HashSet::new();
			for _ in 0..3 {
				set.extend(deck.draw(4));
			}
			
			assert_eq!(set, deck.cards.clone().into_iter().collect());
		}
	}

	#[test]
	fn test_deck_wont_redraw() {
		let mut deck = Deck::<i32>::default();
		deck.add(1);
		deck.add(2);
		assert_ne!(deck.draw_once(), deck.draw_once());
	}

	#[test]
	fn test_deck_wont_draw_discarded() {
		let mut deck = Deck::<i32>::default();
		deck.add(1);
		deck.add(2);
		let drawn = deck.draw_once();
		deck.discard(&[drawn]);
		assert_ne!(deck.draw_once(), drawn);
	}

	#[test]
	fn test_deck_reuse_discarded() {
		let mut deck = Deck::<i32>::default();
		deck.add(1);
		deck.add(2);
		deck.draw_once();
		let drawn = deck.draw_once();
		deck.discard(&[drawn]);
		assert_eq!(deck.draw_once(), drawn);
	}

	#[test]
	fn test_deck_reset() {
		let mut deck = Deck::<i32>::default();
		deck.add(1);
		deck.add(2);
		deck.draw_once();
		deck.reset();
		let mut drawn = deck.draw(2);
		drawn.sort();
		assert_eq!(drawn, vec![1, 2]);
	}

	#[test]
	fn test_deck_reset_discard() {
		let mut deck = Deck::<i32>::default();
		deck.add(1);
		deck.add(2);
		let card = deck.draw_once();
		deck.discard(&[card]);
		deck.reset();
		let mut drawn = deck.draw(2);
		deck.discard(&drawn);  // This used to fail
	}
}
