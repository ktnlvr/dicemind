use crate::parser::{BinaryOperator, Expression, Integer, PositiveInteger};

pub trait Visitor<T> {
    fn visit(&mut self, expr: Expression) -> T {
        match expr {
            Expression::Dice { amount, power } => self.visit_dice(amount, power),
            Expression::Binop { operator, lhs, rhs } => {
                let lhs = self.visit(*lhs);
                let rhs = self.visit(*rhs);

                self.visit_binop(operator, lhs, rhs)
            }
            Expression::Constant(c) => self.visit_constant(c),
            Expression::Subexpression(e) => self.visit(*e),
        }
    }

    fn visit_dice(&mut self, amount: Option<PositiveInteger>, power: Option<PositiveInteger>) -> T;

    fn visit_constant(&mut self, c: Integer) -> T;

    fn visit_binop(&mut self, op: BinaryOperator, lhs: T, rhs: T) -> T;
}
