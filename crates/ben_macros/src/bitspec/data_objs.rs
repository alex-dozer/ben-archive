#[derive(Debug)]
pub struct ParsedPredicate {
    pub field_id: String,
    pub rule_name: String,
    pub op_tokens: proc_macro2::TokenStream,
}

#[derive(Debug)]
pub struct ParsedThreshold {
    pub field_id: String,
    pub rule_name: String,
    pub fact: Option<String>,
    pub flags: Vec<(String, Vec<String>)>,
    pub op: Option<String>,
    pub levels: Vec<(String, f64)>,
}
