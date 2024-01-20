use crate::parser::{BinaryOperator, Expression, Integer, PositiveInteger};

pub trait Visitor<T> {
    fn visit(&mut self, expr: Expression) -> T {
        use Expression::*;

        match expr {
            Dice { count, power } => self.visit_dice(count, power),
            Binop { operator, lhs, rhs } => {
                let lhs = self.visit(*lhs);
                let rhs = self.visit(*rhs);

                self.visit_binop(operator, lhs, rhs)
            }
            Constant(c) => self.visit_constant(c),
            Subexpression(box e) => self.visit_subexpression(e),
            UnaryNegation(box UnaryNegation(box v)) => self.visit(v),
            UnaryNegation(box v) => {
                let v = self.visit(v);
                self.visit_negation(v)
            }
        }
    }

    fn visit_negation(&mut self, value: T) -> T;

    fn visit_dice(&mut self, count: Option<Integer>, power: Option<PositiveInteger>) -> T;

    fn visit_constant(&mut self, c: Integer) -> T;

    fn visit_binop(&mut self, op: BinaryOperator, lhs: T, rhs: T) -> T;

    fn visit_subexpression(&mut self, subexpr: Expression) -> T {
        self.visit(subexpr)
    }
}
