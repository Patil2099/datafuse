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

use std::task::Context;
use std::task::Poll;

use common_datablocks::DataBlock;
use common_exception::Result;
use common_progress::ProgressCallback;
use common_progress::ProgressValues;
use futures::Stream;
use pin_project_lite::pin_project;

use crate::SendableDataBlockStream;

pin_project! {
    pub struct ProgressStream {
        #[pin]
        input: SendableDataBlockStream,
        callback: ProgressCallback,
    }
}

impl ProgressStream {
    pub fn try_create(input: SendableDataBlockStream, callback: ProgressCallback) -> Result<Self> {
        Ok(Self { input, callback })
    }
}

impl Stream for ProgressStream {
    type Item = Result<DataBlock>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.input.poll_next(ctx) {
            Poll::Ready(x) => match x {
                Some(result) => match result {
                    Ok(block) => {
                        let progress_values = ProgressValues {
                            read_rows: block.num_rows(),
                            read_bytes: block.memory_size(),
                            total_rows_to_read: 0,
                        };

                        (this.callback)(&progress_values);
                        Poll::Ready(Some(Ok(block)))
                    }
                    Err(e) => Poll::Ready(Some(Err(e))),
                },
                None => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
