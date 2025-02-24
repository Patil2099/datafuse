// Copyright 2020 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use common_datavalues::DataField;
use common_datavalues::DataSchemaRef;
use common_datavalues::DataSchemaRefExt;
use common_datavalues::DataType;

use crate::PlanNode;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ExplainType {
    Syntax,
    Graph,
    Pipeline,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct ExplainPlan {
    pub typ: ExplainType,
    pub input: Arc<PlanNode>,
}

impl ExplainPlan {
    pub fn schema(&self) -> DataSchemaRef {
        DataSchemaRefExt::create(vec![DataField::new("explain", DataType::Utf8, false)])
    }

    pub fn set_input(&mut self, node: &PlanNode) {
        self.input = Arc::new(node.clone());
    }
}
