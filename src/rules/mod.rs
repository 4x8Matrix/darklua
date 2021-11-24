//! A module that contains the different rules that mutates a Lua block.

mod call_parens;
mod compute_expression;
mod convert_index_to_field;
mod empty_do;
mod group_local;
mod inject_value;
mod method_def;
mod no_local_function;
mod path_import;
mod remove_comments;
mod remove_spaces;
mod rename_variables;
mod unused_if_branch;
mod unused_while;

pub use call_parens::*;
pub use compute_expression::*;
pub use convert_index_to_field::*;
pub use empty_do::*;
pub use group_local::*;
pub use inject_value::*;
pub use method_def::*;
pub use no_local_function::*;
pub use path_import::*;
pub use remove_comments::*;
pub use remove_spaces::*;
pub use rename_variables::*;
pub use unused_if_branch::*;
pub use unused_while::*;

use crate::nodes::Block;

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RobloxLocation {
    Service { service: String, path: PathBuf },
    RelativeToLibrary { relative_path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResolverConfig {
    RobloxLibrary {
        library_root: PathBuf,
        external_dependencies: RobloxLocation,
        // aliases: { map names to location }
    },
}

/// In order to be able to weakly-type the properties of any rule, this enum makes it possible to
/// easily use serde to gather the value associated with a property.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RulePropertyValue {
    Boolean(bool),
    String(String),
    Usize(usize),
    Float(f64),
    StringList(Vec<String>),
    Resolver(ResolverConfig),
    None,
}

/// When implementing the configure method of the Rule trait, the method returns a result that uses
/// this error type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RuleConfigurationError {
    /// When a rule gets an unknown property. The string should be the unknown field name.
    UnexpectedProperty(String),
    /// When a rule has a required property. The string should be the field name.
    MissingProperty(String),
    /// When a property is associated with something else than an expected string. The string is
    /// the property name.
    StringExpected(String),
    /// When a property is associated with something else than an expected unsigned number. The
    /// string is the property name.
    UsizeExpected(String),
    /// When a property is associated with something else than an expected list of strings. The
    /// string is the property name.
    StringListExpected(String),
    /// When the value type is invalid. The string is the property name that was given the wrong
    /// value type.
    UnexpectedValueType(String),
    /// When a specific value is not expected. The suggestion field should provide a message with
    /// examples of accepted values.
    UnexpectedValue {
        property: String,
        value: String,
        suggestion: Option<String>,
    },
}

impl fmt::Display for RuleConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuleConfigurationError::*;

        match self {
            UnexpectedProperty(property) => write!(f, "unexpected field '{}'", property),
            MissingProperty(property) => write!(f, "missing required field '{}'", property),
            StringExpected(property) => write!(f, "string value expected for field '{}'", property),
            UsizeExpected(property) => {
                write!(f, "unsigned integer expected for field '{}'", property)
            }
            StringListExpected(property) => {
                write!(f, "list of string expected for field '{}'", property)
            }
            UnexpectedValueType(property) => write!(f, "unexpected type for field '{}'", property),
            UnexpectedValue {
                property,
                value,
                suggestion,
            } => {
                if let Some(suggestion) = suggestion {
                    write!(
                        f,
                        "unexpected value for field '{}', received '{}' ({})",
                        property, value, suggestion
                    )
                } else {
                    write!(
                        f,
                        "unexpected value for field '{}', received '{}'",
                        property, value
                    )
                }
            }
        }
    }
}

pub type RuleProperties = HashMap<String, RulePropertyValue>;

/// The intent of this struct is to hold data shared across all rules applied to a file.
#[derive(Debug, Clone)]
pub struct Context {
    current_file_path: PathBuf,
}

impl Context {
    pub fn new(current_file_path: PathBuf) -> Self {
        Self { current_file_path }
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            current_file_path: PathBuf::new(),
        }
    }
}

pub type RuleProcessResult = Result<(), Vec<String>>;

/// Defines an interface that will be used to mutate blocks and how to serialize and deserialize
/// the rule configuration.
pub trait Rule: RuleConfiguration {
    /// This method should mutate the given block to apply the rule.
    fn process(&self, block: &mut Block, context: &mut Context) -> RuleProcessResult;
}

pub trait RuleConfiguration {
    /// The rule deserializer will construct the default rule and then send the properties through
    /// this method to modify the behavior of the rule.
    fn configure(&mut self, properties: RuleProperties) -> Result<(), RuleConfigurationError>;
    /// This method should return the unique name of the rule.
    fn get_name(&self) -> &'static str;
    /// For implementing the serialize trait on the Rule trait, this method should return all
    /// properties that differs from their default value.
    fn serialize_to_properties(&self) -> RuleProperties;
    /// Returns `true` if the rule has at least one property.
    fn has_properties(&self) -> bool {
        !self.serialize_to_properties().is_empty()
    }
}

pub trait FlawlessRule {
    fn flawless_process(&self, block: &mut Block, context: &mut Context);
}

impl<T: FlawlessRule + RuleConfiguration> Rule for T {
    fn process(&self, block: &mut Block, context: &mut Context) -> RuleProcessResult {
        self.flawless_process(block, context);
        Ok(())
    }
}

/// A function to get the default rule stack for darklua. All the rules here must preserve all the
/// functionalities of the original code after being applied. They must guarantee that the
/// processed block will work as much as the original one.
pub fn get_default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(RemoveSpaces::default()),
        Box::new(RemoveComments::default()),
        Box::new(ComputeExpression::default()),
        Box::new(RemoveUnusedIfBranch::default()),
        Box::new(RemoveUnusedWhile::default()),
        Box::new(RemoveEmptyDo::default()),
        Box::new(RemoveMethodDefinition::default()),
        Box::new(ConvertIndexToField::default()),
        Box::new(ConvertLocalFunctionToAssign::default()),
        Box::new(GroupLocalAssignment::default()),
        Box::new(RenameVariables::default()),
        Box::new(RemoveFunctionCallParens::default()),
    ]
}

impl FromStr for Box<dyn Rule> {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let rule: Box<dyn Rule> = match string {
            COMPUTE_EXPRESSIONS_RULE_NAME => Box::new(ComputeExpression::default()),
            CONVERT_INDEX_TO_FIELD_RULE_NAME => Box::new(ConvertIndexToField::default()),
            CONVERT_LOCAL_FUNCTION_TO_ASSIGN_RULE_NAME => {
                Box::new(ConvertLocalFunctionToAssign::default())
            }
            GROUP_LOCAL_ASSIGNMENT => Box::new(GroupLocalAssignment::default()),
            INJECT_GLOBAL_VALUE_RULE_NAME => Box::new(InjectGlobalValue::default()),
            REMOVE_COMMENTS_RULE_NAME => Box::new(RemoveComments::default()),
            REMOVE_EMPTY_DO_RULE_NAME => Box::new(RemoveEmptyDo::default()),
            REMOVE_FUNCTION_CALL_PARENS => Box::new(RemoveFunctionCallParens::default()),
            REMOVE_METHOD_DEFINITION_RULE_NAME => Box::new(RemoveMethodDefinition::default()),
            REMOVE_SPACES_RULE_NAME => Box::new(RemoveSpaces::default()),
            REMOVE_UNUSED_IF_BRANCH_RULE_NAME => Box::new(RemoveUnusedIfBranch::default()),
            REMOVE_UNUSED_WHILE_RULE_NAME => Box::new(RemoveUnusedWhile::default()),
            RENAME_VARIABLES_RULE_NAME => Box::new(RenameVariables::default()),
            _ => return Err(format!("invalid rule name: {}", string)),
        };

        Ok(rule)
    }
}

impl Serialize for Box<dyn Rule> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let properties = self.serialize_to_properties();
        let property_count = properties.len();
        let rule_name = self.get_name();

        if property_count == 0 {
            serializer.serialize_str(rule_name)
        } else {
            let mut map = serializer.serialize_map(Some(property_count + 1))?;

            map.serialize_entry("rule", rule_name)?;

            let mut ordered: Vec<(String, RulePropertyValue)> = properties.into_iter().collect();

            ordered.sort_by(|a, b| a.0.cmp(&b.0));

            for (key, value) in ordered {
                map.serialize_entry(&key, &value)?;
            }

            map.end()
        }
    }
}

impl<'de> Deserialize<'de> for Box<dyn Rule> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Box<dyn Rule>, D::Error> {
        struct StringOrStruct;

        impl<'de> Visitor<'de> for StringOrStruct {
            type Value = Box<dyn Rule>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("rule name or rule object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut rule: Self::Value = FromStr::from_str(value).map_err(de::Error::custom)?;

                rule.configure(RuleProperties::new())
                    .map_err(de::Error::custom)?;

                Ok(rule)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut rule_name = None;
                let mut properties = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rule" => {
                            if rule_name.is_none() {
                                rule_name.replace(map.next_value::<String>()?);
                            } else {
                                return Err(de::Error::duplicate_field("rule"));
                            }
                        }
                        property => {
                            let value = map.next_value::<RulePropertyValue>()?;

                            if properties.insert(property.to_owned(), value).is_some() {
                                return Err(de::Error::custom(format!(
                                    "duplicate field {} in rule object",
                                    property
                                )));
                            }
                        }
                    }
                }

                if let Some(rule_name) = rule_name {
                    let mut rule: Self::Value =
                        FromStr::from_str(&rule_name).map_err(de::Error::custom)?;

                    rule.configure(properties).map_err(de::Error::custom)?;

                    Ok(rule)
                } else {
                    Err(de::Error::missing_field("rule"))
                }
            }
        }

        deserializer.deserialize_any(StringOrStruct)
    }
}

fn verify_no_rule_properties(properties: &RuleProperties) -> Result<(), RuleConfigurationError> {
    if let Some((key, _value)) = properties.iter().next() {
        return Err(RuleConfigurationError::UnexpectedProperty(key.to_owned()));
    }
    Ok(())
}

fn verify_required_rule_properties(
    properties: &RuleProperties,
    names: &[&str],
) -> Result<(), RuleConfigurationError> {
    for name in names.iter() {
        if !properties.contains_key(*name) {
            return Err(RuleConfigurationError::MissingProperty(name.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use insta::assert_json_snapshot;

    #[test]
    fn snapshot_default_rules() {
        let rules = get_default_rules();

        assert_json_snapshot!("default_rules", rules);
    }

    #[test]
    fn verify_no_rule_properties_is_ok_when_empty() {
        let empty_properties = RuleProperties::default();

        assert_eq!(verify_no_rule_properties(&empty_properties), Ok(()));
    }

    #[test]
    fn verify_no_rule_properties_is_unexpected_rule_err() {
        let mut properties = RuleProperties::default();
        let some_rule_name = "rule name";
        properties.insert(some_rule_name.to_owned(), RulePropertyValue::None);

        assert_eq!(
            verify_no_rule_properties(&properties),
            Err(RuleConfigurationError::UnexpectedProperty(
                some_rule_name.to_owned()
            ))
        );
    }
}
