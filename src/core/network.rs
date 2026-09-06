pub(crate) async fn fetch_items() -> Result<Vec<crate::model::Item>, String> {
    // TODO: implement HTTP fetch
    Ok(Vec::new())
}

pub(crate) async fn sync_item(_item: &crate::model::Item) -> Result<(), String> {
    // TODO: implement HTTP sync
    Ok(())
}
