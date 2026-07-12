use parallax_proto::semconv;

pub(crate) fn escape(text: &str) -> String {
    text.replace('\'', "''")
}

pub(crate) fn escape_ident(text: &str) -> String {
    text.replace('"', "\"\"")
}

pub(crate) fn quoted_ident(text: &str) -> String {
    format!(r#""{}""#, escape_ident(text))
}

pub(crate) fn resource_attr_ident(attribute: &str) -> String {
    quoted_ident(&semconv::resource_column(attribute))
}

pub(crate) fn wire_attr_ident(attribute: &str) -> String {
    quoted_ident(attribute)
}

pub(crate) fn resource_json_get(attribute: &str) -> String {
    format!(
        r#"json_get_string("resource_attributes", '{}')"#,
        semconv::resource_json_path(attribute)
    )
}

pub(crate) fn log_service_name_expr() -> String {
    format!(
        r#"COALESCE("service.name", {})"#,
        resource_json_get(semconv::SERVICE_NAME)
    )
}
