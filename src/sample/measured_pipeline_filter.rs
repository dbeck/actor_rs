
use lossyq::spsc::Sender;
use super::super::elem::filter;
use super::super::{ChannelWrapper, Message, Schedule};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct MeasuredPipelineFilter {
  on_exec:  u64,
  on_msg:   u64,
  latency:  u64,
  spinned:  Arc<AtomicUsize>,
}

impl filter::Filter for MeasuredPipelineFilter {
  type InputType  = usize;
  type OutputType = usize;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputType>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule
  {
    self.on_exec += 1;
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for m in receiver.iter() {
        self.on_msg += 1;
        if let Message::Value(tick) = m {
          let now = self.spinned.load(Ordering::Acquire);
          self.latency += (now - tick as usize) as u64;
        }
        output.put(|v| *v = Some(m));
      }
      // only execute when there is a new message on the input channel
      Schedule::OnMessage(*channel_id)
      //Schedule::Loop
    } else {
      Schedule::Stop
    }
  }
}

impl MeasuredPipelineFilter {
  pub fn new(spinned: Arc<AtomicUsize>) -> MeasuredPipelineFilter {
    MeasuredPipelineFilter{
      on_exec: 0,
      on_msg:  0,
      latency: 0,
      spinned: spinned,
    }
  }
}

pub fn new(spinned: Arc<AtomicUsize>) -> MeasuredPipelineFilter {
  MeasuredPipelineFilter::new(spinned)
}

impl Drop for MeasuredPipelineFilter {
  fn drop(&mut self) {
    println!(" @drop MeasuredPipelineFilter exec_count:{} msg_count:{} avg latency {} spins",
      self.on_exec,
      self.on_msg,
      self.latency/self.on_msg
    );
  }
}
