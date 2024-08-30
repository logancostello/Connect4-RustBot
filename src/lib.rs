use std::path::Path;
use std::fs;
use std::time::{Duration, SystemTime};
use rand::prelude::*;
use std::cmp::Ordering;

struct Position {
    board: [u64; 2], // stores bitboards for each player
    turn: usize, // tracks which player it is to play
    moves: Vec<usize>, // history of moves
    height_mask: u64, // tracks next playable spot for each column
}

impl Position {

    // returns height mask for a specific column
    pub fn get_col_height_mask(&self, col: usize) -> u64 {
        let col_mask: u64 = 0b1111111;
        self.height_mask & col_mask << (7 * col)
    }

    // given a column, returns if it can be played in or not
    pub fn is_legal_move(&self, col: usize) -> bool {
        let top_row: u64 = 0b1000000100000010000001000000100000010000001000000;
        self.get_col_height_mask(col) | top_row != top_row
    }

    // receives a column to play in (assumed to be legal) and plays it
    pub fn make_move(&mut self, col: usize) {
        let move_mask = self.get_col_height_mask(col);
        self.board[self.turn] |= move_mask;
        self.turn = 1 - self.turn;
        self.moves.push(col);
        self.height_mask ^= move_mask | move_mask << 1;
    }

    // play multiple moves. used for tests
    pub fn make_moves(&mut self, cols: Vec<usize>) {
        for col in cols {
            self.make_move(col);
        }
    }

    // undo a move
    pub fn undo_move(&mut self) {
        let last_col = self.moves.pop().unwrap();
        let undo_mask = self.get_col_height_mask(last_col) >> 1;
        self.height_mask ^= undo_mask | undo_mask << 1;
        self.turn = 1 - self.turn;
        self.board[self.turn] ^= undo_mask;
    }

    // check if a move results in connect 4
    pub fn is_winning_move(&self, col: usize) -> bool {
        let b = self.board[self.turn] | self.get_col_height_mask(col);
        if ((b & b << 1 & b << 2 & b << 3) |
            (b & b << 7 & b << 14 & b << 21) |
            (b & b << 6 & b << 12 & b << 18) |
            (b & b << 8 & b << 16 & b << 24)) != 0 {
                return true
            } 
        false
    }

    // check if a move opens an opportunity for the opponent to win
    pub fn is_losing_move(&self, col: usize, threats: u64) -> bool {
        threats & self.get_col_height_mask(col) << 1 > 0
    }

    // returns col of must play move, all otherwise
    pub fn must_play_move(&self, live_threats: u64) -> usize {
        if live_threats == 0 { return 7 }
        (live_threats.ilog2() / 7) as usize
    }

    // check if position is a guarenteed loss (assumes can't win this turn)
    pub fn is_losing_position(&self, threats: u64, live_threats: u64) -> bool {
        let stacked_threats = threats & threats >> 1;

        if u64::count_ones(live_threats) > 1 { return true };
        if live_threats & stacked_threats > 0 { return true };
        false
    }

    // get opponents live threats
    pub fn get_live_threats(&self, threats: u64) -> u64 {
        let board = self.board[0] | self.board[1] | 283691315109952; 
        (threats & board << 1) | (threats & 1) // & 1 since no bit can be undo bit 0
    }

    // get unique key that represents the position
    pub fn hash(&self) -> u64 {
        self.board[self.turn] | self.height_mask
    }
}

// gets threats of opponent
fn get_threats(board: [u64; 2], player: usize) -> u64 {
    let open: u64 = !(board[0] | board[1] | 283691315109952 | 71776119061217280);
    let opp: u64 = board[player];

    let mut threats: u64 = 0;

    // vertical threats
    threats |= open & opp << 1 & opp << 2 & opp << 3;

    // horizontal threats
    threats |= open & opp << 7 & opp << 14 & opp << 21;
    threats |= opp >> 7 & open & opp << 7 & opp << 14;
    threats |= opp >> 7 & opp >> 14 & open & opp << 7;
    threats |= opp >> 7 & opp >> 14 & opp >> 21 & open;
    
    // positive diagonol threats
    threats |= open & opp << 8 & opp << 16 & opp << 24;
    threats |= opp >> 8 & open & opp << 8 & opp << 16;
    threats |= opp >> 16 & opp >> 8 & open & opp << 8;
    threats |= opp >> 24 & opp >> 16 & opp >> 8 & open;

    // negative diagonol threats
    threats |= open & opp << 6 & opp << 12 & opp << 18;
    threats |= opp >> 6 & open & opp << 6 & opp << 12;
    threats |= opp >> 12 & opp >> 6 & open & opp << 6;
    threats |= opp >> 18 & opp >> 12 & opp >> 6 & open;

    threats
}

// creates empty transposition table
fn create_tt() -> Box<[u64; 1000000]> {
    // the tt stores u64s, so 64 bits of information
    // Bits 0-48 hold the key, to confirm we are colliding while searching
    // Bit 49-50 hold the alphabeta flag. 00 for lowerbound, 01 for exact, 10 for upperbound
    // Bit 51 holds the sign of the score. 1 is negative
    // Bits 52-56 holds the absolute value of the score
    // Bits 57-63 are unused
    Box::new([0; 1000000])
}

// takes a position, iteratively deepens to find its score, returns score and # positions searched
fn score(pos: &mut Position) -> (i8, u64) {
    // null window search window = [alpha, alpha + 1]
    // two return options, score > alpha or score <= alpha
    // by using a small window, we quickly determine if the true score is better or worse than alpha
    // we then adjust our window towards to direction of the true score

    // counterintuitively these many searches are faster than a single search due to the small window resulting
    // in a lot of pruning, and the transposition table helps us not repeat searches

    let mut min: i8 = -1 * ((42 - pos.moves.len()) / 2) as i8;
    let mut max: i8 = ((43 - pos.moves.len()) / 2) as i8;
    let mut tt = create_tt();
    let mut positions_searched: u64 = 0;

    while min < max {
        let mut med: i8 = min + (max - min) / 2;
        if med <= 0 && min / 2 < med {med = min / 2}
        else if med >= 0 && max / 2 > med {med = max / 2}

        let (s, p) = negamax(pos, med, med + 1, &mut tt);
        positions_searched += p;

        if s <= med {max = s}
        else {min = s}
    }
    (min, positions_searched)
}

// takes a position, does a negamax search, returns its score and how many positions were searched
fn negamax(pos: &mut Position, mut alpha: i8, mut beta: i8, tt: &mut Box<[u64; 1000000]>) -> (i8, u64) {

    // use prior search if one exists
    let hash = pos.hash();
    let tt_record: u64 = tt[(pos.hash() % 1000000) as usize];
    if hash == (tt_record & (2_u64.pow(49) - 1)) { // confirm record is for the position we are searching
        let flag = tt_record >> 49 & 0b11;
        let mut score: i8 = (tt_record >> 52 & 0b1111) as i8;
        if (tt_record >> 51 & 0b1) == 1 { score *= -1 };
        if flag == 0b00 { // lowerbound
            if score > alpha { alpha = score }
        } else if flag == 0b01 { // exact
            return (score, 0);
        } else { // upperbound
            if score < beta { beta = score }
        }

        if alpha >= beta { return (alpha, 0) };
    }

    // track original alpha for storing in transposition table
    let original_alpha = alpha;

    // track # positions searched
    // unnecessary for scoring a position, but useful for tracking progress
    let mut total_positions: u64 = 1;

    // check if game is a tie
    if pos.moves.len() == 42 {return (0, total_positions)};

    // sort moves to optimize pruning
    let mut move_options = [3, 2, 4, 1, 5, 0, 6];
    move_options.sort_by(|a, b| order_moves(a, b, &pos));

    // check for a winning move
    for mv in move_options {
        if pos.is_legal_move(mv) && pos.is_winning_move(mv) {
            return ((43 - pos.moves.len() as i8) / 2, total_positions);
        }
    }

    // get threats for various reasons
    let threats = get_threats(pos.board, 1 - pos.turn);
    let live_threats = pos.get_live_threats(threats);

    // check if the position is a loss on opponents next turn (since we cannot win on this turn)
    if pos.is_losing_position(threats, live_threats) { return ((-42 + pos.moves.len() as i8) / 2, total_positions) }

    // beta should be <= the max possible score
    let max_possible_score: i8 = (41 - pos.moves.len() as i8) / 2;
    if beta > max_possible_score {
        beta = max_possible_score;
        if alpha >= beta { return (beta, total_positions) } // alpha beta window is empty
    }

    // if there is a must play move, it is our only option
    let must_play_move = pos.must_play_move(live_threats);
    if must_play_move < 7 {
        // ideally we could update move options to just have the must play move, but 
        // when move options is a vec! it is much slower than when it is an array
        pos.make_move(must_play_move);
        let (s, p) = negamax(pos, -1 * beta, -1 * alpha, tt);
        pos.undo_move();
        total_positions += p;
        alpha = -1 * s
    } else {
        // search all legal moves 
        for mv in move_options {
            if pos.is_legal_move(mv) && !pos.is_losing_move(mv, threats) {
                pos.make_move(mv);
                let (mut s, p) = negamax(pos, -1 * beta, -1 * alpha, tt);
                pos.undo_move();
                s *= -1; 
                total_positions += p;
                if s > alpha { alpha = s };
                if alpha >= beta { break }
            }
        }
    }

    // store score in transposition table for lookup if position is searched again
    let mut into_tt: u64 = hash; // record key
    if alpha < 0 { into_tt |= 1 << 51 } // record sign
    into_tt |= (alpha.abs() as u64) << 52;// record abs of score

    if alpha <= original_alpha { // upperbound
        into_tt |= 0b10 << 49;
    } else if alpha >= beta { // lowerbound
        into_tt |= 0b00 << 49;
    } else { // exact
        into_tt |= 0b01 << 49;
    }
    tt[(hash % 1000000) as usize] = into_tt; // store the value

    (alpha, total_positions)
}

// compares two moves, determines which should be search first
// looking at better moves first results in more pruning from alpha-beta, so we guess which moves will be best
fn order_moves(a: &usize, b: &usize, pos: &Position) -> Ordering {
    // moves closer to the center are generally better, but they are already column sorted

    // moves that create threats are, on average, better than moves that don't
    let mut board_a = pos.board;
    board_a[pos.turn] |= pos.get_col_height_mask(*a);

    let mut board_b = pos.board;
    board_b[pos.turn] |= pos.get_col_height_mask(*b);

    let threats_from_a = get_threats(board_a, pos.turn).count_ones();
    let threats_from_b = get_threats(board_b, pos.turn).count_ones();
    
    if threats_from_a > threats_from_b {
        Ordering::Less
    } else if threats_from_a == threats_from_b {
        Ordering::Equal
    } else {
        Ordering::Greater
    }    
}


#[cfg(test)]
mod tests {
    use super::*;

    // returns start position
    fn start_position() -> Position {
        Position {
            board: [0, 0],
            turn: 0,
            moves: Vec::new(),
            height_mask: 0b0000001000000100000010000001000000100000010000001
        }
    }

    // takes a valid test position key and turns it into a position
    fn key_to_position(key: String) -> Position {
        let mut p = start_position();
        for keymove in key.chars() {
            // test keys use 1-7, i use 0-6
            p.make_move(keymove.to_digit(10).unwrap() as usize - 1);
        }
        p
    }

    // takes file path, tests score func on those position
    // returns [% correct, avg solve time, avg # of positions searched]
    fn check_progress<P: AsRef<Path>>(file_path: P) -> (f64, f64, u64) {
        // read the desired file
        let contents = match fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error reading file: {}", e); 
                return (-1.0, -1.0, 1)
            },
        };

        let mut num_correct: f64 = 0.0;
        let mut total_time = Duration::new(0, 0);
        let mut total_num_positions: u64 = 0;

        let mut line_count = 0;
        //for each line, test the score func
        for line in contents.lines() {
            line_count += 1;
            let mut parts = line.split_whitespace();
            let key = parts.next().unwrap();
            let actual_score: i8 = parts.next().unwrap().parse().expect("Not a valid number");

            let mut p = key_to_position(key.to_string());
            let start = SystemTime::now();
            let (predicted_score, num_positions) = score(&mut p);
            let end = SystemTime::now();
            
            let duration = end.duration_since(start).expect("Time went backwards");
            if actual_score == predicted_score { 
                num_correct += 1.0;
                println!("{line_count}: {} {key}", duration.as_secs()); 
            } else {
                println!("INCORRECT {line_count}: {} {key} {actual_score} {predicted_score}", duration.as_secs());
            };

            total_time += duration;
            total_num_positions += num_positions;
        }
        // return progress information
        (num_correct / 10.0, (total_time.as_micros() / 1000) as f64 / 1_000_000.0, total_num_positions/ 1000)
    }

    #[test]
    fn test_make_move_0() {
        let mut p = start_position();

        p.make_move(0);
        assert_eq!(p.board[0], 1);
        assert_eq!(p.board[1], 0);
        assert_eq!(p.height_mask, 0b0000001000000100000010000001000000100000010000010);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![0]);
    }

    #[test]
    fn test_make_move_1() {
        let mut p = start_position();

        p.make_move(3);
        assert_eq!(p.board[0], u64::pow(2, 21));
        assert_eq!(p.board[1], 0);
        assert_eq!(p.height_mask, 0b0000001000000100000010000010000000100000010000001);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![3]);
    }

    #[test]
    fn test_make_move_2() {
        let mut p = start_position();

        p.make_moves(vec![0, 0, 0]);
        assert_eq!(p.moves, vec![0, 0, 0]);
        assert_eq!(p.board, [5, 2]);
        assert_eq!(p.turn, 1);
        assert_eq!(p.height_mask, 0b0000001000000100000010000001000000100000010001000);
    }
    

    #[test]
    fn test_undo_move_0() {
        let mut p = start_position();
        
        p.make_move(1);
        p.undo_move();

        assert_eq!(p.board, [0, 0]);
        assert_eq!(p.turn, 0);
    }
 
    #[test]
    fn test_undo_move_1() {
        let mut p = start_position();

        p.make_moves(vec![1, 2, 4, 6, 2, 2, 2, 1]);
        for _i in 0..8 {
            p.undo_move();
        }

        assert_eq!(p.board, [0, 0]);
        assert_eq!(p.turn, 0);
        assert_eq!(p.moves, vec![]);
        assert_eq!(p.height_mask, 0b0000001000000100000010000001000000100000010000001);
    }
    

    #[test]
    fn test_is_legal_move_0() {
        let mut p = start_position();
        p.make_moves(vec![3, 3, 3, 3, 3, 3, 5, 5, 5, 5, 5, 5, 1, 2, 4, 6, 1, 1, 1]);
        
        assert_eq!(p.is_legal_move(0), true);
        assert_eq!(p.is_legal_move(1), true);
        assert_eq!(p.is_legal_move(2), true);
        assert_eq!(p.is_legal_move(3), false);
        assert_eq!(p.is_legal_move(4), true);
        assert_eq!(p.is_legal_move(5), false);
        assert_eq!(p.is_legal_move(6), true);
    }
    
    #[test]
    fn test_is_winning_move_0() { // horizontal
        let mut p = start_position();
        p.make_moves(vec![3, 3, 2, 2, 4, 4]);
        assert_eq!(p.is_winning_move(5), true);
    }

    #[test]
    fn test_is_winning_move_1() { // vertical
        let mut p = start_position();
        p.make_moves(vec![3, 2, 3, 2, 3, 2, 0]);
        assert_eq!(p.is_winning_move(2), true);
    }

    #[test]
    fn test_is_winning_move_2() { // positive diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 1, 1, 2, 2, 3, 2, 3, 3, 4]);
        assert_eq!(p.is_winning_move(3), true);
    }

    #[test]
    fn test_is_winning_move_3() { // negative diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 6, 5, 5, 4, 4, 3, 4, 3, 3, 2]);
        assert_eq!(p.is_winning_move(3), true);
    }

    #[test]
    fn test_is_winning_move_4() { // vertical wrapping 
        let mut p = start_position();
        p.make_moves(vec![0, 0, 0, 0, 3, 0, 3, 0, 3]);
        assert_eq!(p.is_winning_move(1), false);
    }

    #[test]
    fn test_key_to_position_0() {
        let p = key_to_position(String::from("111"));

        assert_eq!(p.board, [5, 2]);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![0, 0, 0]);
        assert_eq!(p.height_mask, 0b0000001000000100000010000001000000100000010001000);
    }

    #[test]
    fn test_score_0() { // will be a tie in 1 move
        let mut pos = key_to_position(String::from("11111122222234333334444455555567676776767"));
        let (s, _p) = score(&mut pos);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_score_1() { // will be a tie in 5 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556767677"));
        let (s, _p) = score(&mut pos);
        assert_eq!(s, 0);
    }

    #[test]
    fn test_score_2() { // player 1 can win in 2 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556767"));
        let (s, _p) = score(&mut pos);
        assert_eq!(s, 3);
    }

    #[test]
    fn test_score_3() { // player 1 loses in 3 moves
        let mut pos = key_to_position(String::from("1111112222223433333444445555556766"));
        let (s, _p) = score(&mut pos);
        assert_eq!(s, -2);
    }

    #[test]
    fn test_score_4() { // player 2 can win in 4 moves
        let mut pos = key_to_position(String::from("111111222222343333344444555555676"));
        let (s, _p) = score(&mut pos);
        assert_eq!(s, 2);
    }
    
    #[test]
    fn test_threats_0() { // horizontal
        let mut pos = start_position();
        pos.make_moves(vec![1, 1, 2, 2, 3, 3]);

        assert_eq!(get_threats(pos.board, 1 - pos.turn), 2 + 2_u64.pow(29));
    }

    #[test]
    fn test_threats_1() { // vertical
        let mut pos = start_position();
        pos.make_moves(vec![0, 1, 0, 1, 0, 1]);

        assert_eq!(get_threats(pos.board, 1 - pos.turn), 2_u64.pow(10));

        pos.make_moves(vec![1, 0, 5, 6, 5, 6]);
        assert_eq!(get_threats(pos.board, 1 - pos.turn), 0);
    }

    #[test]
    fn test_threats_2() { // positive diagonol
        let mut pos = start_position();
        pos.make_moves(vec![1, 1, 2, 3, 2, 2, 3, 3, 6, 3]);

        assert_eq!(get_threats(pos.board, 1 - pos.turn), 1 + 2_u64.pow(32));
    } 

    #[test]
    fn test_threats_3() { // negative diagonol
        let mut pos = start_position();
        pos.make_moves(vec![5, 5, 4, 3, 4, 4, 3, 3, 1, 3]);

        assert_eq!(get_threats(pos.board, 1 - pos.turn), 2_u64.pow(42) + 2_u64.pow(18));
    } 

    #[test]
    fn is_losing_position_0() {
        let mut p = start_position();
        p.make_moves(vec![2, 2, 3, 3, 4]);
        let threats = get_threats(p.board, 1 - p.turn);
        let live = p.get_live_threats(threats);

        assert_eq!(p.is_losing_position(threats, live), true)
    }

    #[test]
    fn is_losing_position_1() {
        let mut p = start_position();
        p.make_moves(vec![1, 6, 1, 6, 2, 5, 2, 4, 3, 4, 3]);
        let threats = get_threats(p.board, 1 - p.turn);
        let live = p.get_live_threats(threats);

        assert_eq!(p.is_losing_position(threats, live), true);
    }

    #[test]
    fn is_losing_move_0() {
        let mut p = start_position();
        p.make_moves(vec![0, 2, 0, 2, 3, 3, 4, 4]);

        let threats = get_threats(p.board, 1 - p.turn);

        assert_eq!(p.is_losing_move(5, threats), true);
        assert_eq!(p.is_losing_move(1, threats), true);
        assert_eq!(p.is_losing_move(6, threats), false);
    }

    #[test]
    fn is_losing_move_1() {
        let mut p = start_position();
        p.make_moves(vec![0, 1, 0, 1, 0, 1, 1, 0, 2, 1, 2, 2, 2, 2, 3, 2, 3, 3, 3, 3, 4, 5]);

        let threats = get_threats(p.board, 1 - p.turn);
        
        assert_eq!(p.is_losing_move(4, threats), true);
        assert_eq!(p.is_losing_move(3, threats), false);
    }

    #[test]
    fn test_hash_0() {
        let mut p = start_position();
        assert_eq!(p.hash(), 0b1000000100000010000001000000100000010000001);

        p.make_move(3);
        assert_eq!(p.hash(), 0b1000000100000010000010000000100000010000001);

        p.make_move(3);
        assert_eq!(p.hash(), 0b1000000100000010000101000000100000010000001);
    }

    #[test]
    fn test_hash_1() {
        let mut p = start_position();
        p.make_moves(vec![0, 0, 0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 6, 6, 6, 6, 6, 6]);
        assert_eq!(p.hash(), 0b1010101000000100000100110101001010100001010001010);
    }

    #[test]
    fn test_hash_2() {
        let mut p = start_position();
        p.make_moves(vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3]);
        assert_eq!(p.hash(), 0b0000001000000100000011000000111111110000001111111);
    }

    #[test]
    fn test_order_moves_0() {
        let p = start_position();
        let mut moves = [3, 2, 4, 1, 5, 0, 6];
        moves.sort_by(|a, b| order_moves(a, b, &p));

        assert_eq!(moves, [3, 2, 4, 1, 5, 0, 6])   
    }

    #[test]
    fn test_order_moves_1() {
        let mut p = start_position();
        p.make_moves(vec![2, 2, 3, 3]);
        
        let mut moves = [3, 2, 4, 1, 5, 0, 6];
        moves.sort_by(|a, b| order_moves(a, b, &p));

        assert_eq!(moves, [4, 1, 5, 0, 3, 2, 6]);
    }

    #[test]
    fn test_order_moves_2() {
        let mut p = start_position();
        p.make_moves(vec![0, 0, 1, 1, 5, 5, 4, 4]);
        
        let mut moves = [3, 2, 4, 1, 5, 0, 6];
        moves.sort_by(|a, b| order_moves(a, b, &p));

        assert_eq!(moves, [3, 2, 6, 4, 1, 5, 0]);
    }

    #[test]
    fn test_progress_check() { // used to check efficiency progress, will not pass
        let result = check_progress("test_files/Start-Hard.txt");
        assert_eq!(result, (0.0, 0.0, 0));
    }    
}
