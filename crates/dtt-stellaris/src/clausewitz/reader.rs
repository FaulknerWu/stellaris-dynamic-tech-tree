use dtt_core::condition::ComparisonOperator;
use jomini::Utf8Encoding;
use jomini::text::{Operator, TextToken, ValueReader};

use super::document::Utf8Object;
use crate::error::Error;

pub(crate) type Utf8Value<'data, 'tokens> = ValueReader<'data, 'tokens, Utf8Encoding>;

pub(crate) fn map_operator(operator: Option<Operator>) -> ComparisonOperator {
    match operator {
        None | Some(Operator::Equal) => ComparisonOperator::Equal,
        Some(Operator::LessThan) => ComparisonOperator::LessThan,
        Some(Operator::LessThanEqual) => ComparisonOperator::LessThanOrEqual,
        Some(Operator::GreaterThan) => ComparisonOperator::GreaterThan,
        Some(Operator::GreaterThanEqual) => ComparisonOperator::GreaterThanOrEqual,
        Some(Operator::NotEqual) => ComparisonOperator::NotEqual,
        Some(Operator::Exact) => ComparisonOperator::Exact,
        Some(Operator::Exists) => ComparisonOperator::Exists,
    }
}

pub(crate) fn read_scalar(value: &Utf8Value<'_, '_>) -> Result<String, Error> {
    value.read_string().map_err(Error::from)
}

pub(crate) fn read_object<'data, 'tokens>(
    value: &Utf8Value<'data, 'tokens>,
) -> Result<Utf8Object<'data, 'tokens>, Error> {
    value.read_object().map_err(Error::from)
}

pub(crate) fn read_scalar_values(value: &Utf8Value<'_, '_>) -> Result<Vec<String>, Error> {
    if is_scalar_token(value) {
        return Ok(vec![read_scalar(value)?]);
    }
    let array = value.read_array().map_err(Error::from)?;
    let mut values = Vec::new();
    for entry in array.values() {
        values.push(read_scalar(&entry)?);
    }
    Ok(values)
}

pub(crate) fn is_scalar_token(value: &Utf8Value<'_, '_>) -> bool {
    matches!(
        value.token(),
        TextToken::Unquoted(_) | TextToken::Quoted(_) | TextToken::Header(_)
    )
}

pub(crate) fn is_array_token(value: &Utf8Value<'_, '_>) -> bool {
    matches!(value.token(), TextToken::Array { .. })
}
