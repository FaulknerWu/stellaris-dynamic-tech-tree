use dtt_core::condition::ComparisonOperator;
use jomini::text::TextToken;

use super::document::Utf8Object;
use super::reader::{
    Utf8Value, is_array_token, is_scalar_token, map_operator, read_object, read_scalar,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CwObject {
    pub entries: Vec<CwEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CwEntry {
    Field(CwField),
    Value(CwValue),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CwField {
    pub key: String,
    pub operator: ComparisonOperator,
    pub value: CwValue,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CwValue {
    Scalar(String),
    Object(CwObject),
    Array(Vec<CwValue>),
    Malformed,
}

impl CwObject {
    pub(crate) fn from_utf8(object: &Utf8Object<'_, '_>) -> Self {
        Self {
            entries: collect_sequence(object)
                .into_iter()
                .map(sequence_entry_to_cw)
                .collect(),
        }
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = &CwField> {
        self.entries.iter().filter_map(|entry| match entry {
            CwEntry::Field(field) => Some(field),
            CwEntry::Value(_) => None,
        })
    }

    pub(crate) fn first_field(&self, name: &str) -> Option<&CwField> {
        self.fields().find(|field| field.key == name)
    }

    pub(crate) fn first_scalar(&self, name: &str) -> Option<&str> {
        match self.first_field(name).map(|field| &field.value) {
            Some(CwValue::Scalar(value)) => Some(value.as_str()),
            _ => None,
        }
    }

    pub(crate) fn as_object_value(value: &CwValue) -> Option<&CwObject> {
        match value {
            CwValue::Object(object) => Some(object),
            _ => None,
        }
    }
}

impl CwValue {
    pub(crate) fn from_utf8(value: &Utf8Value<'_, '_>) -> Self {
        if is_scalar_token(value) {
            return match read_scalar(value) {
                Ok(scalar) => Self::Scalar(scalar),
                Err(_) => Self::Malformed,
            };
        }
        if let Ok(object) = read_object(value) {
            return Self::Object(CwObject::from_utf8(&object));
        }
        if is_array_token(value)
            && let Ok(array) = value.read_array()
        {
            return Self::Array(
                array
                    .values()
                    .map(|entry| Self::from_utf8(&entry))
                    .collect(),
            );
        }
        Self::Malformed
    }

    pub(crate) fn as_scalar(&self) -> Option<&str> {
        match self {
            Self::Scalar(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

enum SequenceEntry<'data, 'tokens> {
    Field {
        key: String,
        operator: ComparisonOperator,
        value: Utf8Value<'data, 'tokens>,
    },
    Value(Utf8Value<'data, 'tokens>),
}

fn sequence_entry_to_cw(entry: SequenceEntry<'_, '_>) -> CwEntry {
    match entry {
        SequenceEntry::Field {
            key,
            operator,
            value,
        } => CwEntry::Field(CwField {
            key,
            operator,
            value: CwValue::from_utf8(&value),
        }),
        SequenceEntry::Value(value) => CwEntry::Value(CwValue::from_utf8(&value)),
    }
}

fn collect_sequence<'data, 'tokens>(
    object: &Utf8Object<'data, 'tokens>,
) -> Vec<SequenceEntry<'data, 'tokens>> {
    let mut fields_iter = object.fields();
    let mut entries = Vec::new();
    for (key, operator, value) in fields_iter.by_ref() {
        entries.push(SequenceEntry::Field {
            key: key.read_string(),
            operator: map_operator(operator),
            value,
        });
    }

    let mut remainder = fields_iter.remainder().values().peekable();
    let mut in_mixed_fields = !entries.is_empty();
    while let Some(value) = remainder.next() {
        if matches!(value.token(), TextToken::MixedContainer) {
            in_mixed_fields = true;
            continue;
        }

        if in_mixed_fields
            && let Some(TextToken::Operator(operator)) = remainder.peek().map(|value| value.token())
        {
            let Ok(key) = read_scalar(&value) else {
                entries.push(SequenceEntry::Value(value));
                continue;
            };
            let operator = *operator;
            remainder.next();
            let Some(field_value) = remainder.next() else {
                break;
            };
            entries.push(SequenceEntry::Field {
                key,
                operator: map_operator(Some(operator)),
                value: field_value,
            });
            continue;
        }

        if matches!(value.token(), TextToken::Operator(_)) {
            continue;
        }
        entries.push(SequenceEntry::Value(value));
    }
    entries
}

pub(crate) fn substitute_placeholders(
    value: &str,
    parameters: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let mut remaining = value;
    let mut output = String::with_capacity(value.len());

    while let Some(start) = remaining.find('$') {
        output.push_str(&remaining[..start]);
        remaining = &remaining[start + 1..];
        let end = remaining.find('$')?;
        let placeholder = &remaining[..end];
        let (parameter, default) = match placeholder.split_once('|') {
            Some((parameter, default)) => (parameter, Some(default)),
            None => (placeholder, None),
        };
        if parameter.is_empty() {
            return None;
        }
        match parameters.get(parameter) {
            Some(bound) => output.push_str(bound),
            None => output.push_str(default?),
        }
        remaining = &remaining[end + 1..];
    }

    output.push_str(remaining);
    Some(output)
}

pub(crate) fn resolve_at_variable(
    raw: &str,
    variables: &std::collections::HashMap<String, String>,
) -> String {
    match raw.strip_prefix('@') {
        Some(name) => variables
            .get(name)
            .cloned()
            .unwrap_or_else(|| raw.to_string()),
        None => raw.to_string(),
    }
}

pub(crate) fn apply_placeholders_to_object(
    object: &CwObject,
    parameters: &std::collections::HashMap<String, String>,
) -> CwObject {
    CwObject {
        entries: object
            .entries
            .iter()
            .map(|entry| match entry {
                CwEntry::Field(field) => CwEntry::Field(CwField {
                    key: substitute_placeholders(&field.key, parameters)
                        .unwrap_or_else(|| field.key.clone()),
                    operator: field.operator,
                    value: apply_placeholders_to_value(&field.value, parameters),
                }),
                CwEntry::Value(value) => {
                    CwEntry::Value(apply_placeholders_to_value(value, parameters))
                }
            })
            .collect(),
    }
}

fn apply_placeholders_to_value(
    value: &CwValue,
    parameters: &std::collections::HashMap<String, String>,
) -> CwValue {
    match value {
        CwValue::Scalar(scalar) => CwValue::Scalar(
            substitute_placeholders(scalar, parameters).unwrap_or_else(|| scalar.clone()),
        ),
        CwValue::Object(object) => {
            CwValue::Object(apply_placeholders_to_object(object, parameters))
        }
        CwValue::Array(values) => CwValue::Array(
            values
                .iter()
                .map(|item| apply_placeholders_to_value(item, parameters))
                .collect(),
        ),
        CwValue::Malformed => CwValue::Malformed,
    }
}
