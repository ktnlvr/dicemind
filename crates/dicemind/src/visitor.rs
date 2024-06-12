use smallvec::SmallVec;

use crate::syntax::{AnnotationString, Augmentation, BinaryOperator, Expression, Integer};

pub trait Visitor<T> {
    fn visit(&mut self, expr: Expression) -> T {
        use Expression::*;

        match expr {
            Dice {
                quantity,
                power,
                augmentations,
            } => {
                let quantity = quantity.map(|e| self.visit(*e)).unwrap_or_else(|| self.default_quantity());
                let power = power.map(|e| self.visit(*e)).unwrap_or_else(|| self.default_power());

                self.visit_dice(quantity, power, augmentations)
            }
            Binop { operator, lhs, rhs } => {
                let lhs = self.visit(*lhs);
                let rhs = self.visit(*rhs);

                self.visit_binop(operator, lhs, rhs)
            }
            Constant(c) => self.visit_constant(c),
            Subexpression(box e) => self.visit_subexpression(e),
            Annotated {
                expression: box expr,
                annotation,
            } => self.visit_annotated(expr, annotation),
            UnaryNegation(box UnaryNegation(box v)) => self.visit(v),
            UnaryNegation(box v) => {
                let v = self.visit(v);
                self.visit_negation(v)
            }
        }
    }

    fn visit_negation(&mut self, value: T) -> T;

    fn visit_dice(
        &mut self,
        quantity: T,
        power: T,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> T;

    fn visit_constant(&mut self, c: Integer) -> T;

    fn visit_binop(&mut self, op: BinaryOperator, lhs: T, rhs: T) -> T;

    fn visit_annotated(&mut self, expr: Expression, _annotation: AnnotationString) -> T {
        self.visit(expr)
    }

    fn visit_subexpression(&mut self, subexpr: Expression) -> T {
        self.visit(subexpr)
    }

    fn default_quantity(&self) -> T;

    fn default_power(&self) -> T;
}
