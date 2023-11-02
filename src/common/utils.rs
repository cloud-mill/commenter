use uuid::Uuid;

pub fn uuid_list_to_materialized_path(uuids: &[Uuid]) -> String {
    uuids
        .iter()
        .map(|uuid| uuid.to_string())
        .collect::<Vec<String>>()
        .join("->")
}

pub fn append_uuid_to_materialized_path(existing_path: &str, new_uuid: &Uuid) -> String {
    if existing_path.is_empty() {
        new_uuid.to_string()
    } else {
        format!("{}->{}", existing_path, new_uuid)
    }
}
