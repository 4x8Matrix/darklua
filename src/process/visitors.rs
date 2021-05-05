use crate::nodes::*;
use crate::process::NodeProcessor;

use std::marker::PhantomData;

/// A trait that defines method that iterates on nodes and process them using a NodeProcessor.
pub trait NodeVisitor<T: NodeProcessor> {
    fn visit_block(block: &mut Block, processor: &mut T) {
        processor.process_block(block);

        block.mutate_statements()
            .iter_mut()
            .for_each(|statement| Self::visit_statement(statement, processor));

        if let Some(last_statement) = block.mutate_last_statement() {
            processor.process_last_statement(last_statement);

            match last_statement {
                LastStatement::Return(expressions) => {
                    expressions.iter_mut()
                        .for_each(|expression| Self::visit_expression(expression, processor));
                }
                _ => {}
            };
        };
    }

    fn visit_statement(statement: &mut Statement, processor: &mut T) {
        processor.process_statement(statement);

        match statement {
            Statement::Assign(statement) => Self::visit_assign_statement(statement, processor),
            Statement::Do(statement) => Self::visit_do_statement(statement, processor),
            Statement::Call(statement) => Self::visit_function_call(statement, processor),
            Statement::Function(statement) => Self::visit_function_statement(statement, processor),
            Statement::GenericFor(statement) => Self::visit_generic_for(statement, processor),
            Statement::If(statement) => Self::visit_if_statement(statement, processor),
            Statement::LocalAssign(statement) => Self::visit_local_assign(statement, processor),
            Statement::LocalFunction(statement) => Self::visit_local_function(statement, processor),
            Statement::NumericFor(statement) => Self::visit_numeric_for(statement, processor),
            Statement::Repeat(statement) => Self::visit_repeat_statement(statement, processor),
            Statement::While(statement) => Self::visit_while_statement(statement, processor),
        };
    }

    fn visit_expression(expression: &mut Expression, processor: &mut T) {
        processor.process_expression(expression);

        match expression {
            Expression::Binary(expression) => {
                processor.process_binary_expression(expression);
                Self::visit_expression(expression.mutate_left(), processor);
                Self::visit_expression(expression.mutate_right(), processor);
            }
            Expression::Call(expression) => Self::visit_function_call(expression, processor),
            Expression::Field(field) => Self::visit_field_expression(field, processor),
            Expression::Function(function) => Self::visit_function_expression(function, processor),
            Expression::Identifier(identifier) => processor.process_variable_expression(identifier),
            Expression::Index(index) => Self::visit_index_expression(index, processor),
            Expression::Number(number) => processor.process_number_expression(number),
            Expression::Parenthese(expression) => Self::visit_expression(expression, processor),
            Expression::String(string) => processor.process_string_expression(string),
            Expression::Table(table) => Self::visit_table(table, processor),
            Expression::Unary(unary) => {
                processor.process_unary_expression(unary);
                Self::visit_expression(unary.mutate_expression(), processor);
            }
            Expression::LUX(lux) => Self::visit_lux_expression(lux, processor),
            Expression::False
            | Expression::Nil
            | Expression::True
            | Expression::VariableArguments => {}
        }
    }

    fn visit_function_expression(function: &mut FunctionExpression, processor: &mut T) {
        processor.process_function_expression(function);

        Self::visit_block(function.mutate_block(), processor);
    }

    fn visit_assign_statement(statement: &mut AssignStatement, processor: &mut T) {
        processor.process_assign_statement(statement);

        statement.mutate_variables().iter_mut()
            .for_each(|variable| match variable {
                Variable::Identifier(identifier) => processor.process_variable_expression(identifier),
                Variable::Field(field) => Self::visit_field_expression(field, processor),
                Variable::Index(index) => Self::visit_index_expression(index, processor),
            });

        statement.mutate_values().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
    }

    fn visit_do_statement(statement: &mut DoStatement, processor: &mut T) {
        processor.process_do_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_function_statement(statement: &mut FunctionStatement, processor: &mut T) {
        processor.process_function_statement(statement);
        processor.process_variable_expression(statement.mutate_function_name().mutate_identifier());
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_generic_for(statement: &mut GenericForStatement, processor: &mut T) {
        processor.process_generic_for_statement(statement);

        statement.mutate_expressions().iter_mut()
            .for_each(|expression| Self::visit_expression(expression, processor));
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_if_statement(statement: &mut IfStatement, processor: &mut T) {
        processor.process_if_statement(statement);

        statement.mutate_branches()
            .iter_mut()
            .for_each(|branch| {
                Self::visit_expression(branch.mutate_condition(), processor);
                Self::visit_block(branch.mutate_block(), processor);
            });

        if let Some(block) = statement.mutate_else_block() {
            Self::visit_block(block, processor);
        }
    }

    fn visit_local_assign(statement: &mut LocalAssignStatement, processor: &mut T) {
        processor.process_local_assign_statement(statement);

        statement.mutate_values().iter_mut()
            .for_each(|value| Self::visit_expression(value, processor));
    }

    fn visit_local_function(statement: &mut LocalFunctionStatement, processor: &mut T) {
        processor.process_local_function_statement(statement);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_numeric_for(statement: &mut NumericForStatement, processor: &mut T) {
        processor.process_numeric_for_statement(statement);

        Self::visit_expression(statement.mutate_start(), processor);
        Self::visit_expression(statement.mutate_end(), processor);

        if let Some(step) = statement.mutate_step() {
            Self::visit_expression(step, processor);
        };

        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_repeat_statement(statement: &mut RepeatStatement, processor: &mut T) {
        processor.process_repeat_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_while_statement(statement: &mut WhileStatement, processor: &mut T) {
        processor.process_while_statement(statement);

        Self::visit_expression(statement.mutate_condition(), processor);
        Self::visit_block(statement.mutate_block(), processor);
    }

    fn visit_field_expression(field: &mut FieldExpression, processor: &mut T) {
        processor.process_field_expression(field);

        Self::visit_prefix_expression(field.mutate_prefix(), processor);
    }

    fn visit_index_expression(index: &mut IndexExpression, processor: &mut T) {
        processor.process_index_expression(index);

        Self::visit_prefix_expression(index.mutate_prefix(), processor);
        Self::visit_expression(index.mutate_index(), processor);
    }

    fn visit_function_call(call: &mut FunctionCall, processor: &mut T) {
        processor.process_function_call(call);

        Self::visit_prefix_expression(call.mutate_prefix(), processor);
        Self::visit_arguments(call.mutate_arguments(), processor);
    }

    fn visit_arguments(arguments: &mut Arguments, processor: &mut T) {
        match arguments {
            Arguments::String(string) => processor.process_string_expression(string),
            Arguments::Table(table) => Self::visit_table(table, processor),
            Arguments::Tuple(expressions) => expressions.iter_mut()
                .for_each(|expression| Self::visit_expression(expression, processor)),
        }
    }

    fn visit_table(table: &mut TableExpression, processor: &mut T) {
        processor.process_table_expression(table);

        table.mutate_entries().iter_mut()
            .for_each(|entry| match entry {
                TableEntry::Field(_field, value) => Self::visit_expression(value, processor),
                TableEntry::Index(key, value) => {
                    Self::visit_expression(key, processor);
                    Self::visit_expression(value, processor);
                }
                TableEntry::Value(value) => Self::visit_expression(value, processor),
            });
    }

    fn visit_prefix_expression(prefix: &mut Prefix, processor: &mut T) {
        processor.process_prefix_expression(prefix);

        match prefix {
            Prefix::Call(call) => Self::visit_function_call(call, processor),
            Prefix::Field(field) => Self::visit_field_expression(field, processor),
            Prefix::Identifier(identifier) => processor.process_variable_expression(identifier),
            Prefix::Index(index) => Self::visit_index_expression(index, processor),
            Prefix::Parenthese(expression) => Self::visit_expression(expression, processor),
        };
    }

    fn visit_lux_expression(expression: &mut LUXExpression, processor: &mut T) {
        processor.process_lux_expression(expression);

        match expression {
            LUXExpression::LUXElement(element) => Self::visit_lux_element(element, processor),
            LUXExpression::LUXFragment(fragment) => Self::visit_lux_fragment(fragment, processor),
        }
    }

    fn visit_lux_element(element: &mut LUXElement, processor: &mut T) {
        processor.process_lux_element(element);

        match element {
            LUXElement::Element(element) => {
                processor.process_lux_open_close_element(element);

                element.mutate_attributes().iter_mut()
                    .for_each(|attribute| {
                        Self::visit_lux_attribute(attribute, processor);
                    });

                element.mutate_children().iter_mut()
                    .for_each(|child| {
                        processor.process_lux_child(child);
                        Self::visit_lux_child(child, processor);
                    });
            }
            LUXElement::SelfClosingElement(element) => {
                processor.process_lux_self_closing_element(element);

                element.mutate_attributes().iter_mut()
                    .for_each(|attribute| {
                        Self::visit_lux_attribute(attribute, processor);
                    });
            }
        }
    }

    fn visit_lux_fragment(fragment: &mut LUXFragment, processor: &mut T) {
        processor.process_lux_fragment(fragment);

        fragment.mutate_children().iter_mut()
            .for_each(|child| {
                processor.process_lux_child(child);
                Self::visit_lux_child(child, processor);
            });
    }

    fn visit_lux_child(child: &mut LUXChild, processor: &mut T) {
        processor.process_lux_child(child);

        match child {
            LUXChild::LUXElement(element) => {
                Self::visit_lux_element(element, processor);
            }
            LUXChild::LUXFragment(fragment) => {
                Self::visit_lux_fragment(fragment, processor);
            }
            LUXChild::ExpandedExpression(expression) => {
                Self::visit_expression(expression, processor);
            }
            LUXChild::Expression(Some(expression)) => {
                Self::visit_expression(expression, processor);
            }
            LUXChild::Expression(None) => {}
        }
    }

    fn visit_lux_attribute(attribute: &mut LUXAttribute, processor: &mut T) {
        processor.process_lux_attribute(attribute);

        match attribute {
            LUXAttribute::Named(attribute) => {
                if let Some(value) = attribute.mutate_value() {
                    Self::visit_lux_attribute_value(value, processor);
                }
            }
            LUXAttribute::Spread(expression) => {
                Self::visit_expression(expression, processor);
            }
        }
    }

    fn visit_lux_attribute_value(value: &mut LUXAttributeValue, processor: &mut T) {
        processor.process_lux_attribute_value(value);

        match value {
            LUXAttributeValue::DoubleQuoteString(_string) => {}
            LUXAttributeValue::SingleQuoteString(_string) => {}
            LUXAttributeValue::LuaExpression(expression) => {
                Self::visit_expression(expression, processor);
            }
            LUXAttributeValue::LUXElement(element) => {
                Self::visit_lux_element(element, processor);
            }
            LUXAttributeValue::LUXFragment(fragment) => {
                Self::visit_lux_fragment(fragment, processor);
            }
        }
    }
}

/// The default node visitor.
pub struct DefaultVisitor<T> {
    _phantom: PhantomData<T>,
}

impl<T: NodeProcessor> NodeVisitor<T> for DefaultVisitor<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::process::NodeCounter;

    #[test]
    fn visit_do_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(DoStatement::default());

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.do_count, 1);
    }

    #[test]
    fn visit_numeric_for_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(NumericForStatement::new(
                "i".to_owned(),
                Expression::True,
                Expression::True,
                None,
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 2);
        assert_eq!(counter.numeric_for_count, 1);
    }

    #[test]
    fn visit_generic_for_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(GenericForStatement::new(
                vec!["k".to_owned()],
                vec![Expression::True],
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.generic_for_count, 1);
    }

    #[test]
    fn visit_repeat_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(RepeatStatement::new(
                Block::default(),
                Expression::True,
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.repeat_count, 1);
    }

    #[test]
    fn visit_while_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(WhileStatement::new(
                Block::default(),
                Expression::True,
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.while_count, 1);
    }

    #[test]
    fn visit_if_statement() {
        let mut counter = NodeCounter::new();
        let mut block = Block::default()
            .with_statement(IfStatement::create(
                Expression::True,
                Block::default(),
            ));

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 2);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.if_count, 1);
    }

    #[test]
    fn visit_if_statement_with_else() {
        let mut counter = NodeCounter::new();
        let if_statement = IfStatement::create(Expression::True, Block::default())
            .with_else_block(Block::default());

        let mut block = Block::default().with_statement(if_statement);

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 3);
        assert_eq!(counter.expression_count, 1);
        assert_eq!(counter.if_count, 1);
    }

    #[test]
    fn visit_if_statement_with_elseif_and_else() {
        let mut counter = NodeCounter::new();
        let if_statement = IfStatement::create(Expression::True, Block::default())
            .with_branch(Expression::False, Block::default())
            .with_else_block(Block::default());

        let mut block = Block::default().with_statement(if_statement);

        DefaultVisitor::visit_block(&mut block, &mut counter);

        assert_eq!(counter.block_count, 4);
        assert_eq!(counter.expression_count, 2);
        assert_eq!(counter.if_count, 1);
    }
}
