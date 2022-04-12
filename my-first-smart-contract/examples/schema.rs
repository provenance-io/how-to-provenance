use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use my_first_smart_contract::msg::{ExecuteMsg, InitMsg, QueryMsg};

/// This rust file is used to automatically generate a schema output for all entrypoint values.
/// This is to help users of the contract get an idea of how to format the json used when calling
/// into the instantiate, execute, and query routes.
fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
