use std::ops::Add;

use crate::token::TokenPosition;

#[derive(Debug, PartialEq, Clone)]
pub struct SplitItem {
    pub text: String,
    pub position: TokenPosition
}
impl SplitItem {
    pub fn new(text: String, position: TokenPosition) -> SplitItem {
        SplitItem {
            text,
            position
        }
    }

    pub fn new_pl(text: String, position: i32, line: i32) -> SplitItem {
        SplitItem {
            text,
            position: TokenPosition::new(position, line)
        }
    }
}

impl Add for SplitItem {
    type Output = SplitItem;

    fn add(self, rhs: SplitItem) -> SplitItem {
        SplitItem {
            text: self.text + &rhs.text,
            position: self.position + rhs.position
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let split_item = SplitItem {
            text: "Hello,".to_string(),
            position: TokenPosition::new(0, 0)
        };
        let sum_item = SplitItem {
            text: " World!".to_string(),
            position: TokenPosition::new(0, 0)
        };

        assert_eq!(split_item + sum_item, SplitItem {
            text: "Hello, World!".to_string(),
            position: TokenPosition::new(0, 0)
        });
    }
}