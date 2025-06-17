use color_eyre::eyre::WrapErr;
use ratatui::crossterm::event::{self, Event as CrosstermEvent};
use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
    time::{Duration, Instant},
};

/// All possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Periodic timer event for updating the simulation.
    Tick,
    /// Terminal events (keyboard, mouse, resize).
    Crossterm(CrosstermEvent),
    /// Application events.
    App(AppEvent),
}

/// High-level application events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Reset and randomize the simualation.
    Randomize,
    /// Clear the simulation.
    Clear,
    /// Quit the application.
    Quit,
}

/// Control messages for the event thread
#[derive(Clone, Debug)]
pub enum ControlMessage {
    /// Update the tick interval.
    SetTickInterval(Duration),
    /// Pause tick events.
    Pause,
    /// Resume tick events.
    Resume,
}

/// Manages event collection and distribution.
///
/// Spawns a background thread that:
/// - Polls for terminal events
/// - Generates tick events at configurable intervals
/// - Handles pause/resume functionality
#[derive(Debug)]
pub struct EventHandler {
    /// Channel for sending events to the main thread.
    event_sender: mpsc::Sender<Event>,
    /// Channel for receiving events in the main thread.
    event_receiver: mpsc::Receiver<Event>,
    /// Channel for sending control messages to the event thread.
    control_sender: mpsc::Sender<ControlMessage>,
}

impl EventHandler {
    /// Creates a new event handler and spawns the event collection thread.
    pub fn new(tick_interval: Duration, paused: bool) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        let (control_sender, control_receiver) = mpsc::channel();
        let actor = EventThread::new(
            event_sender.clone(),
            control_receiver,
            tick_interval,
            paused,
        );
        thread::spawn(|| actor.run());
        Self {
            event_sender,
            event_receiver,
            control_sender,
        }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub fn next(&self) -> color_eyre::Result<Event> {
        Ok(self.event_receiver.recv()?)
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.event_sender.send(Event::App(app_event));
    }

    /// Updates the tick event interval.
    pub fn set_tick_interval(&self, interval: Duration) {
        let _ = self
            .control_sender
            .send(ControlMessage::SetTickInterval(interval));
    }

    /// Pauses tick event generation.
    pub fn pause(&self) {
        let _ = self.control_sender.send(ControlMessage::Pause);
    }

    /// Resumes tick event generation.
    pub fn resume(&self) {
        let _ = self.control_sender.send(ControlMessage::Resume);
    }
}

/// Background thread that collects events from multiple sources.
struct EventThread {
    /// Channel for sending events to the main thread.
    event_sender: mpsc::Sender<Event>,
    /// Channel for receiving control messages.
    control_receiver: mpsc::Receiver<ControlMessage>,
    /// Interval between generated tick events.
    tick_interval: Duration,
    /// Whether tick generation is paused.
    paused: bool,
}

impl EventThread {
    /// Creates a new event thread instance.
    fn new(
        event_sender: mpsc::Sender<Event>,
        control_receiver: mpsc::Receiver<ControlMessage>,
        tick_interval: Duration,
        paused: bool,
    ) -> Self {
        Self {
            event_sender,
            control_receiver,
            tick_interval,
            paused,
        }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    fn run(mut self) -> color_eyre::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            // Process all pending control messages without blocking
            loop {
                match self.control_receiver.try_recv() {
                    Ok(msg) => self.handle_control_message(msg),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return Ok(()),
                }
            }

            // Calculate poll timeout
            let elapsed = last_tick.elapsed();
            let time_until_tick = if self.paused {
                // Longer timeout when paused to reduce CPU usage
                Duration::from_millis(100)
            } else {
                self.tick_interval.saturating_sub(elapsed)
            };

            // Generate tick if due
            if !self.paused && time_until_tick == Duration::ZERO {
                last_tick = Instant::now();
                self.send(Event::Tick);
            }

            // Poll for terminal events
            if event::poll(time_until_tick).wrap_err("failed to poll for crossterm events")? {
                let event = event::read().wrap_err("failed to read crossterm event")?;
                self.send(Event::Crossterm(event));
            }
        }
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.event_sender.send(event);
    }

    /// Handle control messages
    fn handle_control_message(&mut self, msg: ControlMessage) {
        match msg {
            ControlMessage::SetTickInterval(interval) => {
                self.tick_interval = interval;
            }
            ControlMessage::Pause => {
                self.paused = true;
            }
            ControlMessage::Resume => {
                self.paused = false;
            }
        }
    }
}
