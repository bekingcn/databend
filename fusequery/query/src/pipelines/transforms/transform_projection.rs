// Copyright 2020-2021 The Datafuse Authors.
//
// SPDX-License-Identifier: Apache-2.0.

use std::any::Any;
use std::sync::Arc;

use common_datablocks::DataBlock;
use common_datavalues::DataSchemaRef;
use common_exception::Result;
use common_streams::SendableDataBlockStream;
use tokio_stream::StreamExt;

use crate::pipelines::processors::EmptyProcessor;
use crate::pipelines::processors::IProcessor;

pub struct ProjectionTransform {
    schema: DataSchemaRef,
    input: Arc<dyn IProcessor>
}

impl ProjectionTransform {
    pub fn try_create(schema: DataSchemaRef) -> Result<Self> {
        Ok(ProjectionTransform {
            schema,
            input: Arc::new(EmptyProcessor::create())
        })
    }
}

#[async_trait::async_trait]
impl IProcessor for ProjectionTransform {
    fn name(&self) -> &str {
        "ProjectionTransform"
    }

    fn connect_to(&mut self, input: Arc<dyn IProcessor>) -> Result<()> {
        self.input = input;
        Ok(())
    }

    fn inputs(&self) -> Vec<Arc<dyn IProcessor>> {
        vec![self.input.clone()]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn execute(&self) -> Result<SendableDataBlockStream> {
        let projected_schema = self.schema.clone();
        let input_stream = self.input.execute().await?;

        let executor = |schema: DataSchemaRef, block: Result<DataBlock>| -> Result<DataBlock> {
            let block = block?;
            let fields = schema.fields();

            let mut columns = Vec::with_capacity(fields.len());
            for field in fields {
                let column = block.try_column_by_name(field.name().as_str())?;
                columns.push(column.clone());
            }
            Ok(DataBlock::create(schema, columns))
        };

        let stream = input_stream
            .filter_map(move |v| executor(projected_schema.clone(), v).map(Some).transpose());
        Ok(Box::pin(stream))
    }
}
