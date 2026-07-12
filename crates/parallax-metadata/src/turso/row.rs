use super::*;

pub(super) fn text(row: &turso::Row, index: usize) -> String {
    match row.get_value(index) {
        Ok(Value::Text(s)) => s,
        _ => String::new(),
    }
}

pub(super) fn opt_text(row: &turso::Row, index: usize) -> Option<String> {
    match row.get_value(index) {
        Ok(Value::Text(s)) => Some(s),
        _ => None,
    }
}

pub(super) fn integer(row: &turso::Row, index: usize) -> i64 {
    match row.get_value(index) {
        Ok(Value::Integer(v)) => v,
        _ => 0,
    }
}

pub(super) fn opt_integer(row: &turso::Row, index: usize) -> Option<i64> {
    match row.get_value(index) {
        Ok(Value::Integer(v)) => Some(v),
        _ => None,
    }
}
