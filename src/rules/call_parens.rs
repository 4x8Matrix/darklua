use crate::nodes::{Arguments, Block, Expression, FunctionCall, StringExpression, TableExpression};
use crate::process::{DefaultVisitorMut, NodeProcessorMut, NodeVisitorMut};
use crate::rules::{Rule, RuleConfigurationError, RuleProperties};

use std::mem;

#[derive(Debug, Clone, Default)]
struct Processor {}

impl NodeProcessorMut for Processor {
    fn process_function_call(&mut self, call: &mut FunctionCall) {
        let new_arguments = match call.mutate_arguments() {
            Arguments::Tuple(expressions) if expressions.len() == 1 => {
                let expression = expressions.iter_mut().next().unwrap();

                match expression {
                    Expression::String(string) => {
                        let mut steal_string = StringExpression::empty();
                        mem::swap(string, &mut steal_string);
                        Some(Arguments::String(steal_string))
                    }
                    Expression::Table(table) => {
                        let mut steal_table = TableExpression::default();
                        mem::swap(table, &mut steal_table);
                        Some(Arguments::Table(steal_table))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        if let Some(new_arguments) = new_arguments {
            *call.mutate_arguments() = new_arguments;
        }
    }
}

pub const REMOVE_FUNCTION_CALL_PARENS: &'static str = "remove_function_call_parens";

/// A rule that removes parentheses when calling functions with a string or a table.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RemoveFunctionCallParens {}

impl Rule for RemoveFunctionCallParens {
    fn process(&self, block: &mut Block) {
        let mut processor = Processor::default();
        DefaultVisitorMut::visit_block(block, &mut processor);
    }

    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError> {
        for (key, _value) in properties {
            return Err(RuleConfigurationError::UnexpectedProperty(key))
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_FUNCTION_CALL_PARENS
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    fn new_rule() -> RemoveFunctionCallParens {
        RemoveFunctionCallParens::default()
    }

    #[test]
    fn serialize_default_rule() {
        let rule: Box<dyn Rule> = Box::new(new_rule());

        assert_json_snapshot!("default_remove_function_call_parens", rule);
    }
}
