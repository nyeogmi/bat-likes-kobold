use crate::consts::N_MOVES;

#[derive(Clone, Copy)]
pub(crate) enum Rotation {
    Straight, Right, Double, Left
}

impl Rotation {
    pub fn rotate_index(&self, index: u8) -> u8 {
        assert!((0..N_MOVES as u8).contains(&index));
        match self {
            Rotation::Straight => index,
            Rotation::Right => [6, 3, 0, 7, 4, 1, 8, 5, 2][index as usize],
            Rotation::Double => 8 - index,
            Rotation::Left => [2, 5, 8, 1, 4, 7, 0, 3, 6][index as usize],
        }
    }

    pub fn derotate_index(&self, index: u8) -> u8{
        match self {
            Rotation::Straight => Rotation::Straight,
            Rotation::Right => Rotation::Left,
            Rotation::Double => Rotation::Double,
            Rotation::Left => Rotation::Right
        }.rotate_index(index)
    }

    pub fn rotate_matrix<T: Copy>(&self, matrix: [T; N_MOVES]) -> [T; N_MOVES] {
        let mut matrix2 = matrix;
        for i in 0..N_MOVES {
            matrix2[self.rotate_index(i as u8) as usize] = matrix[i as usize]
        }
        return matrix2
    }

    pub fn derotate_matrix<T: Copy>(&self, matrix: [T; N_MOVES]) -> [T; N_MOVES] {
        let mut matrix2 = matrix;
        for i in 0..N_MOVES {
            matrix2[self.derotate_index(i as u8) as usize] = matrix[i as usize]
        }
        return matrix2
    }
}

#[test]
fn test_inverses() {
    for i in 0..N_MOVES as u8 {
        assert!(i == Rotation::Straight.rotate_index(i));
        assert!(i == Rotation::Straight.rotate_index(Rotation::Straight.rotate_index(i)));
        assert!(i == Rotation::Right.rotate_index(Rotation::Left.rotate_index(i)));
        assert!(i == Rotation::Double.rotate_index(Rotation::Double.rotate_index(i)));
        assert!(i == Rotation::Left.rotate_index(Rotation::Right.rotate_index(i)));

        assert!(i == Rotation::Straight.derotate_index(Rotation::Straight.rotate_index(i)));
        assert!(i == Rotation::Left.derotate_index(Rotation::Left.rotate_index(i)));
        assert!(i == Rotation::Double.derotate_index(Rotation::Double.rotate_index(i)));
        assert!(i == Rotation::Right.derotate_index(Rotation::Right.rotate_index(i)));

        assert!(i == Rotation::Straight.derotate_index(Rotation::Straight.derotate_index(i)));
        assert!(i == Rotation::Left.derotate_index(Rotation::Right.derotate_index(i)));
        assert!(i == Rotation::Double.derotate_index(Rotation::Double.derotate_index(i)));
        assert!(i == Rotation::Right.derotate_index(Rotation::Left.derotate_index(i)));
    }
}