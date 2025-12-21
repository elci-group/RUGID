// Pomodoro Timer Application
// Backend: Rust timer logic
// Frontend: RUGID UI framework

use rugid::runtime::Runtime;
use rugid::cell::Cell;
use rugid::geometry::VectorRegion;
use rugid::layout::{LayoutNode, LayoutDirection, LayoutConstraint, LayoutSolver};
use rugid::widgets::{ButtonProjector, SliderProjector};
use rugid::platform::{HeadlessPlatform, Platform, PlatformEvent};
use rugid::input::InputEvent;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ============================================
// BACKEND: Timer State Machine
// ============================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimerState {
    Idle,
    Running,
    Paused,
    Break,
}

struct PomodoroBackend {
    state: TimerState,
    work_duration: Duration,
    break_duration: Duration,
    remaining: Duration,
    last_tick: Instant,
    sessions_completed: u32,
}

impl PomodoroBackend {
    fn new() -> Self {
        Self {
            state: TimerState::Idle,
            work_duration: Duration::from_secs(25 * 60), // 25 minutes
            break_duration: Duration::from_secs(5 * 60), // 5 minutes
            remaining: Duration::from_secs(25 * 60),
            last_tick: Instant::now(),
            sessions_completed: 0,
        }
    }

    fn start(&mut self) {
        if self.state == TimerState::Idle || self.state == TimerState::Paused {
            self.state = TimerState::Running;
            self.last_tick = Instant::now();
        }
    }

    fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.state = TimerState::Paused;
        }
    }

    fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.remaining = self.work_duration;
    }

    fn tick(&mut self) {
        if self.state != TimerState::Running {
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if elapsed < self.remaining {
            self.remaining -= elapsed;
        } else {
            // Timer completed
            if self.state == TimerState::Running {
                self.sessions_completed += 1;
                self.state = TimerState::Break;
                self.remaining = self.break_duration;
            } else if self.state == TimerState::Break {
                self.state = TimerState::Idle;
                self.remaining = self.work_duration;
            }
        }
    }

    fn progress(&self) -> f32 {
        let total = if self.state == TimerState::Break {
            self.break_duration
        } else {
            self.work_duration
        };
        1.0 - (self.remaining.as_secs_f32() / total.as_secs_f32())
    }

    fn display_time(&self) -> String {
        let secs = self.remaining.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

// ============================================
// FRONTEND: RUGID UI
// ============================================

fn main() {
    println!("🍅 Pomodoro Timer - RUGID Demo");
    println!("================================");

    // Initialize Backend
    let backend = Arc::new(Mutex::new(PomodoroBackend::new()));

    // Initialize Frontend (Headless for demo)
    let event_queue = Arc::new(Mutex::new(Vec::new()));
    
    struct SharedPlatform {
        queue: Arc<Mutex<Vec<PlatformEvent>>>,
    }
    
    impl Platform for SharedPlatform {
        fn poll_events(&mut self) -> Vec<PlatformEvent> {
            let mut q = self.queue.lock().unwrap();
            let events = q.clone();
            q.clear();
            events
        }
        fn render(&mut self, _svg: String) {}
    }

    let queue_ref = event_queue.clone();
    let platform = Box::new(SharedPlatform { queue: event_queue });
    let mut runtime = Runtime::new(100, platform);

    // --- UI Layout ---
    // 
    // +------------------------+
    // |     POMODORO TIMER     |  <- Header
    // +------------------------+
    // |                        |
    // |       25:00            |  <- Timer Display (Progress Slider)
    // |                        |
    // +------------------------+
    // |  [START] [PAUSE] [RESET] | <- Controls
    // +------------------------+
    // |   Sessions: 0          |  <- Status
    // +------------------------+

    let header_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let timer_display_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let start_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let pause_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let reset_btn_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));
    let status_cell = Cell::new(VectorRegion::new(0.0, 0.0, 0.0, 0.0));

    // Control Row Layout
    let controls_layout = LayoutNode::container(
        LayoutDirection::StackPrimary, // Row
        LayoutConstraint::Fixed(0.15),
        vec![
            LayoutNode::leaf(start_btn_cell.id, LayoutConstraint::Flex(1.0)),
            LayoutNode::leaf(pause_btn_cell.id, LayoutConstraint::Flex(1.0)),
            LayoutNode::leaf(reset_btn_cell.id, LayoutConstraint::Flex(1.0)),
        ]
    );

    // Root Layout (Column)
    let root = LayoutNode::container(
        LayoutDirection::StackSecondary,
        LayoutConstraint::Fixed(1.0),
        vec![
            LayoutNode::leaf(header_cell.id, LayoutConstraint::Fixed(0.15)),
            LayoutNode::leaf(timer_display_cell.id, LayoutConstraint::Flex(1.0)),
            controls_layout,
            LayoutNode::leaf(status_cell.id, LayoutConstraint::Fixed(0.1)),
        ]
    );

    let screen_rect = VectorRegion::new(0.0, 0.0, 1.0, 1.0);
    let layout_map = LayoutSolver::solve(&root, screen_rect);

    // Register Widgets
    let header_id = header_cell.id;
    runtime.register_cell(header_cell, Box::new(ButtonProjector {
        target: header_id,
        geometry: *layout_map.get(&header_id).unwrap(),
        label: "POMODORO".to_string(),
    }), None, None);

    let timer_id = timer_display_cell.id;
    runtime.register_cell(timer_display_cell, Box::new(SliderProjector {
        target: timer_id,
        geometry: *layout_map.get(&timer_id).unwrap(),
        value: 0.0, // Progress
    }), None, None);

    let start_id = start_btn_cell.id;
    runtime.register_cell(start_btn_cell, Box::new(ButtonProjector {
        target: start_id,
        geometry: *layout_map.get(&start_id).unwrap(),
        label: "START".to_string(),
    }), None, None);

    let pause_id = pause_btn_cell.id;
    runtime.register_cell(pause_btn_cell, Box::new(ButtonProjector {
        target: pause_id,
        geometry: *layout_map.get(&pause_id).unwrap(),
        label: "PAUSE".to_string(),
    }), None, None);

    let reset_id = reset_btn_cell.id;
    runtime.register_cell(reset_btn_cell, Box::new(ButtonProjector {
        target: reset_id,
        geometry: *layout_map.get(&reset_id).unwrap(),
        label: "RESET".to_string(),
    }), None, None);

    let status_id = status_cell.id;
    runtime.register_cell(status_cell, Box::new(ButtonProjector {
        target: status_id,
        geometry: *layout_map.get(&status_id).unwrap(),
        label: "Sessions: 0".to_string(),
    }), None, None);

    // ============================================
    // SIMULATION LOOP (Demonstrates Integration)
    // ============================================
    
    println!("\n📌 Simulating 10 ticks (1 second each in accelerated time)...\n");

    // For demo: We'll simulate accelerated time (1 tick = 1 virtual minute)
    {
        let mut b = backend.lock().unwrap();
        b.work_duration = Duration::from_secs(5); // 5 seconds for demo
        b.break_duration = Duration::from_secs(2);
        b.remaining = b.work_duration;
    }

    // Start the timer
    queue_ref.lock().unwrap().push(PlatformEvent::Input(InputEvent::PointerDown)); // Simulate start click
    {
        let mut b = backend.lock().unwrap();
        b.start();
        println!("▶️  Timer Started: {}", b.display_time());
    }

    for t in 0..10 {
        // Backend Tick
        {
            let mut b = backend.lock().unwrap();
            b.tick();
            
            let state_emoji = match b.state {
                TimerState::Idle => "⏹️",
                TimerState::Running => "▶️",
                TimerState::Paused => "⏸️",
                TimerState::Break => "☕",
            };
            
            println!(
                "Tick {:02}: {} {} | Progress: {:.0}% | Sessions: {}",
                t,
                state_emoji,
                b.display_time(),
                b.progress() * 100.0,
                b.sessions_completed
            );
        }

        // Frontend Tick
        let svg = runtime.tick();

        // Save frame
        fs::write(format!("pomodoro_frame_{:02}.svg", t), svg).unwrap();

        // Simulate 1 second passing
        std::thread::sleep(Duration::from_millis(500));
    }

    println!("\n✅ Simulation Complete!");
    println!("📁 SVG frames saved: pomodoro_frame_XX.svg");
    
    // Final state
    let b = backend.lock().unwrap();
    println!("\n📊 Final State:");
    println!("   - State: {:?}", b.state);
    println!("   - Sessions Completed: {}", b.sessions_completed);
    println!("   - Time Remaining: {}", b.display_time());
}
