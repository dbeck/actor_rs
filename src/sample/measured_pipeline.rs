
use super::measured_pipeline_source::MeasuredPipelineSource;
use super::measured_pipeline_filter::MeasuredPipelineFilter;
use super::measured_pipeline_sink::MeasuredPipelineSink;
use super::dummy_source::DummySource;
use super::super::elem::connectable::{Connectable};
use super::super::elem::{source, sink, filter};
use super::super::scheduler::{Scheduler};
use super::super::{TaskId, SchedulingRule};
use std::sync::atomic::{AtomicUsize};
use std::sync::Arc;

#[allow(dead_code)]
pub struct MeasuredPipeline {
  sched:              Scheduler,
  source_id:          TaskId,
  wait_ticket:        u64,
  sent:               u64,
}

impl MeasuredPipeline {
  pub fn new(spinned:  Arc<AtomicUsize>, dummies: usize) -> MeasuredPipeline {
    let mut sched        = Scheduler::new();

    let (source_task, mut source_out) =
      source::new( "Source", 2_000, Box::new(MeasuredPipelineSource::new(spinned.clone())));

    let (mut filter_task, mut filter_out) =
      filter::new( "Filter", 2_000, Box::new(MeasuredPipelineFilter::new(spinned.clone())));

    let mut sink_task =
      sink::new( "Sink", Box::new(MeasuredPipelineSink::new(spinned.clone())));

    filter_task.connect(&mut source_out).unwrap();
    sink_task.connect(&mut filter_out).unwrap();

    let source_id = sched.add_task(source_task, SchedulingRule::OnExternalEvent).unwrap();
    let _filter_id = sched.add_task(filter_task, SchedulingRule::OnMessage);
    for i in 0..dummies {
      let name = format!("Dummy-{}",i);
      let (dummy_task, mut _dummy_out) =
        source::new( name.as_str(), 2_000, Box::new(DummySource::new()));
        let _dummy_id = sched.add_task(dummy_task, SchedulingRule::OnExternalEvent);
    }
    let _sink_id = sched.add_task(sink_task, SchedulingRule::OnMessage);

    MeasuredPipeline {
      sched:              sched,
      source_id:          source_id,
      wait_ticket:        0,
      sent:               0,
    }
  }

  pub fn start(&mut self) {
    self.sched.start_with_threads(1);
  }

  pub fn stop(&mut self) {
    self.sched.stop();
  }

  pub fn notify(&mut self) {
    let source_id = self.source_id;
    if let Err(e) = self.sched.notify(&source_id) {
      println!("couldn't notify {:?}, err: {:?}",source_id, e);
    } else {
      self.sent += 1;
    }
  }

  pub fn wait(&mut self) {
  }

  #[cfg(feature = "printstats")]
  fn print_stats(&self) {
    println!(" @drop MeasuredPipeline tkt:{} sent:{}",
      self.wait_ticket, self.sent);
  }

  #[cfg(not(feature = "printstats"))]
  fn print_stats(&self) {}
}

impl Drop for MeasuredPipeline {
  fn drop(&mut self) {
    self.print_stats();
  }
}
