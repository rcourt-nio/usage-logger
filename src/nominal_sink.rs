use crate::output::OutputSink;
use crate::pipeline::{MetricRecord, MetricValue};
use nominal_streaming::prelude::*;
use nominal_streaming::stream::NominalDatasetStream;
use nominal_streaming::types::ChannelDescriptor;
use std::error::Error;

pub struct NominalSink {
    stream: NominalDatasetStream,
}

impl NominalSink {
    pub fn new(stream: NominalDatasetStream) -> Self {
        Self { stream }
    }
}

impl OutputSink for NominalSink {
    fn name(&self) -> &'static str {
        "nominal"
    }

    fn consume(&mut self, record: &MetricRecord) -> Result<(), Box<dyn Error>> {
        let ts = record.timestamp;

        // Push changed metric samples
        for sample in &record.samples {
            if !sample.changed {
                continue;
            }

            let cd = ChannelDescriptor::new(&sample.path);
            match &sample.value {
                MetricValue::Double(v) => {
                    self.stream.enqueue(
                        &cd,
                        vec![DoublePoint {
                            timestamp: Some(ts.into_timestamp()),
                            value: *v,
                        }],
                    );
                }
                MetricValue::Integer(v) => {
                    self.stream.enqueue(
                        &cd,
                        vec![IntegerPoint {
                            timestamp: Some(ts.into_timestamp()),
                            value: *v,
                        }],
                    );
                }
                MetricValue::Bool(v) => {
                    self.stream.enqueue(
                        &cd,
                        vec![DoublePoint {
                            timestamp: Some(ts.into_timestamp()),
                            value: if *v { 1.0 } else { 0.0 },
                        }],
                    );
                }
                MetricValue::Text(v) => {
                    self.stream.enqueue(
                        &cd,
                        vec![StringPoint {
                            timestamp: Some(ts.into_timestamp()),
                            value: v.clone(),
                        }],
                    );
                }
                MetricValue::Null => {}
            }
        }

        // Push log entries as string channels
        for log in &record.logs {
            let cd = ChannelDescriptor::new(&log.channel);
            self.stream.enqueue(
                &cd,
                vec![StringPoint {
                    timestamp: Some(log.timestamp.into_timestamp()),
                    value: log.message.clone(),
                }],
            );
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        // NominalDatasetStream flushes on drop — nothing extra needed here.
        // The stream's internal batch processor handles flushing.
        Ok(())
    }
}
