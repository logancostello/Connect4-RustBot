struct Position {
    board: [i64; 2],
    turn: usize,
    moves: Vec<usize>,
    heights: [usize; 7]
}

impl Position {

    // given a column, returns if it can be played in or not
    pub fn  is_legal_move(&self, col: usize) -> bool {
        self.heights[col] < 6
    }

    // receives a column to play in (assumed to be legal) and plays it
    pub fn make_move(&mut self, col: usize) {
        let move_mask: i64 = (1 << (7 * col)) << self.heights[col];
        self.board[self.turn] |= move_mask;
        self.turn = 1 - self.turn;
        self.moves.push(col);
        self.heights[col] += 1;
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
        self.heights[last_col] -= 1;
        self.turn = 1 - self.turn;

        let undo_mask: i64 = 1 << (7 * last_col + self.heights[last_col]);
        self.board[self.turn] ^= undo_mask;
    }

    // check for connect 4. only look at the the player who just made a move
    pub fn is_connect_four(&self) -> bool {
        let b = self.board[1 - self.turn];
        if ((b & b << 1 & b << 2 & b << 3) |
            (b & b << 7 & b << 14 & b << 21) |
            (b & b << 6 & b << 12 & b << 18) |
            (b & b << 8 & b << 16 & b << 24)) != 0 {
                return true
            } 
        false
    }
}    

#[cfg(test)]
mod tests {
    use super::*;

    fn start_position() -> Position {
        Position {
            board: [0, 0],
            turn: 0,
            moves: Vec::new(),
            heights: [0; 7]
        }
    }

    #[test]
    fn test_make_move_0() {
        let mut p = start_position();

        p.make_move(0);
        assert_eq!(p.board[0], 1);
        assert_eq!(p.board[1], 0);
        assert_eq!(p.heights[0], 1);
        assert_eq!(p.turn, 1);
        assert_eq!(p.moves, vec![0]);
    }

    #[test]
    fn test_make_move_1() {
        let mut p = start_position();

        p.make_move(3);
        assert_eq!(p.board[0], i64::pow(2, 21));
        assert_eq!(p.board[1], 0);
        assert_eq!(p.heights, [0, 0, 0, 1, 0, 0, 0]);
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
        assert_eq!(p.heights, [3, 0, 0, 0, 0, 0, 0]);
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
        for i in 0..8 {
            p.undo_move();
        }

        assert_eq!(p.board, [0, 0]);
        assert_eq!(p.turn, 0);
        assert_eq!(p.moves, vec![]);
        assert_eq!(p.heights, [0; 7]);
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
    fn test_is_connect_four_0() { // horizontal
        let mut p = start_position();
        p.make_moves(vec![3, 3, 2, 2, 4, 4, 5]);
        assert_eq!(p.is_connect_four(), true);
    }

    #[test]
    fn test_is_connect_four_1() { // vertical
        let mut p = start_position();
        p.make_moves(vec![3, 2, 3, 2, 3, 2, 0, 2]);
        assert_eq!(p.is_connect_four(), true);
    }

    #[test]
    fn test_is_connect_four_2() { // positive diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 1, 1, 2, 2, 3, 2, 3, 3, 4, 3]);
        assert_eq!(p.is_connect_four(), true);
    }

    #[test]
    fn test_is_connect_four_3() { // negative diagonol
        let mut p = start_position();
        p.make_moves(vec![0, 6, 5, 5, 4, 4, 3, 4, 3, 3, 2, 3]);
        assert_eq!(p.is_connect_four(), true);
    }

    #[test]
    fn test_is_connect_four_4() { // vertical wrapping 
        let mut p = start_position();
        p.make_moves(vec![0, 0, 0, 0, 3, 0, 3, 0, 3, 1]);
        assert_eq!(p.is_connect_four(), false);
    }
}

