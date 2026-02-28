// use ben_macros::BenSchema;

// #[derive(Debug, BenSchema)]
// #[bschema(
//     table = "sandbox_event",
//     version = 1,
//     order_by = "id, ts",
//     partition_by = "toYYYYMM(toDateTime(ts/1000))",
//     description = "Sandbox PoC event over QUIC"
// )]
// pub struct SandboxEvent {
//     #[bschema(key)]
//     pub id: u64,

//     #[bschema(key, cardinality = "low")]
//     pub kind: String,

//     pub ts: u64,

//     pub value: f64,

//     pub flag: bool,
// }
