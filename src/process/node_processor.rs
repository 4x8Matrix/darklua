use crate::nodes::*;

/// Used by the NodeVisitor trait, a NodeProcessor object is passed to each node to
/// process each node.
pub trait NodeProcessor {
    fn process_block(&mut self, _: &Block) {}
    fn process_statement(&mut self, _: &Statement) {}

    fn process_function_call(&mut self, _: &FunctionCall) {}

    fn process_assign_statement(&mut self, _: &AssignStatement) {}
    fn process_do_statement(&mut self, _: &DoStatement) {}
    fn process_function_statement(&mut self, _: &FunctionStatement) {}
    fn process_generic_for_statement(&mut self, _: &GenericForStatement) {}
    fn process_if_statement(&mut self, _: &IfStatement) {}
    fn process_last_statement(&mut self, _: &LastStatement) {}
    fn process_local_assign_statement(&mut self, _: &LocalAssignStatement) {}
    fn process_local_function_statement(&mut self, _: &LocalFunctionStatement) {}
    fn process_numeric_for_statement(&mut self, _: &NumericForStatement) {}
    fn process_repeat_statement(&mut self, _: &RepeatStatement) {}
    fn process_while_statement(&mut self, _: &WhileStatement) {}

    fn process_expression(&mut self, _: &Expression) {}

    fn process_binary_expression(&mut self, _: &BinaryExpression) {}
    fn process_field_expression(&mut self, _: &FieldExpression) {}
    fn process_function_expression(&mut self, _: &FunctionExpression) {}
    fn process_variable_expression(&mut self, _: &String) {}
    fn process_index_expression(&mut self, _: &IndexExpression) {}
    fn process_number_expression(&mut self, _: &NumberExpression) {}
    fn process_prefix_expression(&mut self, _: &Prefix) {}
    fn process_parenthese_expression(&mut self, _: &Expression) {}
    fn process_string_expression(&mut self, _: &StringExpression) {}
    fn process_table_expression(&mut self, _: &TableExpression) {}
    fn process_unary_expression(&mut self, _: &UnaryExpression) {}
}

/// Used by the NodeVisitorMut trait, a NodeProcessorMut object is passed to each node to
/// perform mutations.
pub trait NodeProcessorMut {
    fn process_block(&mut self, _: &mut Block) {}
    fn process_statement(&mut self, _: &mut Statement) {}

    fn process_function_call(&mut self, _: &mut FunctionCall) {}

    fn process_assign_statement(&mut self, _: &mut AssignStatement) {}
    fn process_do_statement(&mut self, _: &mut DoStatement) {}
    fn process_function_statement(&mut self, _: &mut FunctionStatement) {}
    fn process_generic_for_statement(&mut self, _: &mut GenericForStatement) {}
    fn process_if_statement(&mut self, _: &mut IfStatement) {}
    fn process_last_statement(&mut self, _: &mut LastStatement) {}
    fn process_local_assign_statement(&mut self, _: &mut LocalAssignStatement) {}
    fn process_local_function_statement(&mut self, _: &mut LocalFunctionStatement) {}
    fn process_numeric_for_statement(&mut self, _: &mut NumericForStatement) {}
    fn process_repeat_statement(&mut self, _: &mut RepeatStatement) {}
    fn process_while_statement(&mut self, _: &mut WhileStatement) {}

    fn process_expression(&mut self, _: &mut Expression) {}

    fn process_binary_expression(&mut self, _: &mut BinaryExpression) {}
    fn process_field_expression(&mut self, _: &mut FieldExpression) {}
    fn process_function_expression(&mut self, _: &mut FunctionExpression) {}
    fn process_variable_expression(&mut self, _: &mut String) {}
    fn process_index_expression(&mut self, _: &mut IndexExpression) {}
    fn process_number_expression(&mut self, _: &mut NumberExpression) {}
    fn process_prefix_expression(&mut self, _: &mut Prefix) {}
    fn process_parenthese_expression(&mut self, _: &mut Expression) {}
    fn process_string_expression(&mut self, _: &mut StringExpression) {}
    fn process_table_expression(&mut self, _: &mut TableExpression) {}
    fn process_unary_expression(&mut self, _: &mut UnaryExpression) {}
}
