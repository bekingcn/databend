//  Copyright 2022 Datafuse Labs.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use std::sync::Arc;

use common_catalog::table::Table;
use common_catalog::table_context::TableContext;
use common_exception::ErrorCode;
use common_exception::Result;
use common_expression::RemoteExpr;
use common_sql::parse_exprs;

// use common_sql::ExpressionParser;
use crate::table_functions::string_value;
use crate::table_functions::TableArgs;
use crate::FuseTable;

pub fn parse_func_table_args(table_args: &TableArgs) -> Result<(String, String)> {
    match table_args {
        // Todo(zhyass): support 3 arguments.
        Some(args) if args.len() == 2 => {
            let db = string_value(&args[0])?;
            let tbl = string_value(&args[1])?;
            Ok((db, tbl))
        }
        _ => Err(ErrorCode::BadArguments(format!(
            "expecting database and table name (as two string literals), but got {:?}",
            table_args
        ))),
    }
}

pub fn get_cluster_keys(
    ctx: Arc<dyn TableContext>,
    table: &FuseTable,
    definition: &str,
) -> Result<(Vec<RemoteExpr<String>>, Option<String>)> {
    let (cluster_keys, plain_keys) = if !definition.is_empty() {
        let table_meta = Arc::new(table.clone());
        let cluster_keys = parse_exprs(ctx, table_meta.clone(), true, definition)?;
        (
            cluster_keys
                .iter()
                .map(|k| {
                    k.project_column_ref(|index| {
                        table_meta.schema().field(*index).name().to_string()
                    })
                    .as_remote_expr()
                })
                .collect(),
            Some(definition.to_string()),
        )
    } else {
        (
            table.cluster_keys(ctx),
            table.cluster_key_meta.as_ref().map(|m| m.1.to_string()),
        )
    };

    if cluster_keys.is_empty() {
        return Err(ErrorCode::InvalidClusterKeys(format!(
            "Invalid clustering keys or table {} is not clustered",
            table.name()
        )));
    }
    Ok((cluster_keys, plain_keys))
}