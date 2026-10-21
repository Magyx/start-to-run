mod audio;
mod schedule;
mod schedule_data;
mod timer;

use std::sync::Arc;

use leptos::prelude::*;
use leptos::*;
use schedule::{IntervalKind, Session};
use timer::{format_time, TimerState};

#[derive(Clone)]
enum Screen {
    WeekList,
    SessionList(u32),
    Timer(Session),
}

#[component]
fn App() -> impl IntoView {
    let (screen, set_screen) = signal(Screen::WeekList);

    view! {
        {move || match screen.get() {
            Screen::WeekList => view! {
                <WeekList on_select=Callback::new(move |w| set_screen.set(Screen::SessionList(w)))/>
            }.into_any(),
            Screen::SessionList(week) => view! {
                <SessionList
                    week=week
                    on_back=move |_| set_screen.set(Screen::WeekList)
                    on_select=Callback::new(move |s| set_screen.set(Screen::Timer(s)))
                />
            }.into_any(),
            Screen::Timer(ref session) => {
                view! {
                    <TimerView
                        session=*session
                        on_back=move |_| set_screen.set(Screen::WeekList)
                    />
                }.into_any()
            }
        }}
    }
}

#[component]
fn WeekList(on_select: Callback<u32>) -> impl IntoView {
    // Group by week, take first session per week for stats.
    let weeks: Vec<(u32, u32, u32)> = (1..=10u32)
        .map(|w| {
            let s = schedule_data::SCHEDULE
                .iter()
                .find(|s| s.week == w)
                .unwrap();
            (w, s.total_secs(), s.run_percentage())
        })
        .collect();

    view! {
        <div class="screen picker">
            <div class="container">
                <h1 class="title">"START TO RUN"</h1>
                <p class="subtitle">"0 → 5K in 10 weeks"</p>

                <div class="week-list">
                    {weeks.into_iter().map(|(w, total, pct)| {
                        let on_click = {
                            move |_| on_select.run(w)
                        };
                        view! {
                            <button class="week-btn" on:click=on_click>
                                <div class="week-badge" style:background=format!(
                                    "linear-gradient(135deg, hsl({}, 70%, 45%), hsl({}, 60%, 35%))",
                                    140 + w * 8, 140 + w * 8
                                )>
                                    {w}
                                </div>
                                <div class="week-info">
                                    <div class="week-name">{format!("Week {}", w)}</div>
                                    <div class="week-meta">
                                        {format!("{} · {}% running", format_time(total), pct)}
                                    </div>
                                </div>
                                <div class="week-bar-track">
                                    <div class="week-bar-fill" style:width=format!("{}%", pct)
                                         style:background=format!("hsl({}, 70%, 50%)", 140 + w * 8)>
                                    </div>
                                </div>
                                <span class="chevron">"›"</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn SessionList(
    week: u32,
    on_back: impl Fn(()) + 'static,
    on_select: Callback<Session>,
) -> impl IntoView {
    let sessions: Vec<&Session> = schedule_data::SCHEDULE
        .iter()
        .filter(|s| s.week == week)
        .collect();

    view! {
        <div class="screen picker">
            <div class="container">
                <button class="back-btn" on:click=move |_| on_back(())>"← All weeks"</button>
                <h2 class="title">{format!("Week {}", week)}</h2>

                <div class="session-list">
                    {sessions.into_iter().map(|s| {
                        let total = s.total_secs();
                        view! {
                            <button class="session-btn" on:click=move |_| on_select.run(*s)>
                                <div class="session-header">
                                    <span class="session-name">{format!("Session {}", s.session)}</span>
                                    <span class="session-time">{format_time(total)}</span>
                                </div>
                                <div class="interval-bar">
                                    {s.intervals.iter().map(|intv| {
                                        let cls = if intv.kind == IntervalKind::Run { "bar-run" } else { "bar-walk" };
                                        view! {
                                            <div class=cls style:flex=format!("{}", intv.duration_secs)>
                                                {if intv.duration_secs >= 60 {
                                                    format!("{}′", intv.duration_secs / 60)
                                                } else {
                                                    String::new()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                                <div class="legend">
                                    <span class="dot-run">"● Run"</span>
                                    <span class="dot-walk">"● Walk"</span>
                                </div>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn TimerView(session: Session, on_back: impl Fn(()) + 'static) -> impl IntoView {
    let (state, set_state) = signal(TimerState::new());
    let (show_time, set_show_time) = signal(true);
    let (volume, set_volume) = signal(0.7_f64);
    let (muted, set_muted) = signal(false);
    let session_rc = Arc::new(session);

    // 1-second tick interval
    {
        let session = session_rc.clone();
        Effect::new(
            move |prev_handle: Option<Option<gloo_timers::callback::Interval>>| {
                // Clear previous interval if any.
                if let Some(Some(_h)) = prev_handle {
                    // drop it
                }

                let st = state.get();
                if st.running && !st.done {
                    let session = session.clone();
                    let handle = gloo_timers::callback::Interval::new(1000, move || {
                        set_state.update(|s| {
                            let changed = s.tick(&session);
                            if changed {
                                if s.done {
                                    audio::finished();
                                } else {
                                    let is_run = s.current_kind(&session) == IntervalKind::Run;
                                    audio::cue(is_run);
                                }
                            }
                        });
                    });
                    Some(handle)
                } else {
                    None
                }
            },
        );
    }

    let session_toggle = session_rc.clone();
    let on_toggle = move |_: web_sys::MouseEvent| {
        audio::init(); // unlock speech/audio from this user tap
        set_state.update(|s| {
            let starting = !s.running && !s.done && s.total_elapsed == 0;
            s.toggle();
            if starting {
                // Announce the first interval right away.
                audio::cue(s.current_kind(&session_toggle) == IntervalKind::Run);
            }
        });
    };

    let on_reset = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        set_state.update(|s| s.reset());
        on_back(());
    };

    let toggle_time = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        set_show_time.update(|v| *v = !*v);
    };

    let on_volume = move |e: web_sys::Event| {
        e.stop_propagation();
        let Ok(value) = event_target_value(&e).parse::<f64>() else {
            return;
        };
        set_volume.set(value);
        set_muted.set(false);
        audio::set_volume(value);
    };

    let toggle_mute = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next_muted = !muted.get();
        set_muted.set(next_muted);
        audio::set_volume(if next_muted { 0.0 } else { volume.get() });
    };

    let s_bg_run = session_rc.clone();
    let s_bg_walk = session_rc.clone();
    let s_label = session_rc.clone();
    let s_action = session_rc.clone();
    let s_countdown = session_rc.clone();
    let s_progress = session_rc.clone();
    let s_blocks = session_rc.clone();
    let s_total = session_rc.clone();
    let s_next = session_rc.clone();

    view! {
        <div class="screen timer-screen"
             class:bg-run=move || !state.get().done && state.get().current_kind(&s_bg_run) == IntervalKind::Run
             class:bg-walk=move || !state.get().done && state.get().current_kind(&s_bg_walk) == IntervalKind::Walk
             class:bg-done=move || state.get().done
             on:click=on_toggle
        >
            // Top bar
            <div class="timer-top">
                <button class="top-btn" on:click=on_reset>"← Back"</button>
                <span class="timer-label">
                    {move || format!("W{} · S{}", s_label.week, s_label.session)}
                </span>
                <button class="top-btn" on:click=toggle_time>
                    {move || if show_time.get() { "⏱ On" } else { "⏱ Off" }}
                </button>
            </div>

            <div class="volume-control" on:click=|e: web_sys::MouseEvent| e.stop_propagation()>
                <button class="top-btn" on:click=toggle_mute>
                    {move || if muted.get() { "Unmute" } else { "Mute" }}
                </button>
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    aria-label="Cue volume"
                    prop:value=move || volume.get().to_string()
                    on:input=on_volume
                />
            </div>

            // Main area
            <div class="timer-main">
                <div class="action-label">
                    {move || {
                        let st = state.get();
                        if st.done {
                            "DONE!".to_string()
                        } else if st.current_kind(&s_action) == IntervalKind::Run {
                            "RUN".to_string()
                        } else {
                            "WALK".to_string()
                        }
                    }}
                </div>

                <Show when=move || show_time.get() && !state.get().done>
                    <div class="countdown">
                        { let s_countdown = s_countdown.clone(); move || format_time(state.get().interval_remaining(&s_countdown))}
                    </div>
                    <div class="countdown-sub">"remaining in interval"</div>
                </Show>

                <Show when=move || !state.get().running && !state.get().done && state.get().total_elapsed == 0>
                    <div class="hint">"Tap to start"</div>
                </Show>

                <Show when=move || !state.get().running && !state.get().done && (state.get().total_elapsed > 0)>
                    <div class="hint pulse">"⏸ PAUSED"</div>
                </Show>

                <Show when=move || state.get().done>
                    <div class="hint done-msg">"Well done! 🎉"</div>
                </Show>
            </div>

            // Bottom
            <div class="timer-bottom">
                // Progress bar
                <div class="progress-track">
                    <div class="progress-fill"
                         style:width=move || format!("{}%", state.get().progress(&s_progress) * 100.0)>
                    </div>
                </div>

                // Interval blocks
                <div class="interval-blocks">
                    {s_blocks.intervals.iter().enumerate().map(|(i, intv)| {
                        let kind = intv.kind;
                        let dur = intv.duration_secs;
                        view! {
                            <div
                                style:flex=format!("{}", dur)
                                class="block"
                                class:block-run=kind == IntervalKind::Run
                                class:block-walk=kind == IntervalKind::Walk
                                class:block-past=move || i < state.get().current_index
                                class:block-current=move || i == state.get().current_index && !state.get().done
                            />
                        }
                    }).collect_view()}
                </div>

                <Show when=move || show_time.get()>
                    {
                        let s_total = s_total.clone();
                        let s_next = s_next.clone();
                        view! {
                            <div class="timer-footer">
                                <span>{move || format!("Total: {}", format_time(state.get().total_remaining(&s_total))) }</span>
                                <span>
                                    {move || {
                                        match state.get().next_interval(&s_next) {
                                            Some((kind, dur)) => format!(
                                                "Next: {} {}",
                                                if kind == IntervalKind::Run { "Run" } else { "Walk" },
                                                format_time(dur),
                                            ),
                                            None => "Last interval".to_string(),
                                        }
                                    }}
                                </span>
                            </div>
                        }
                    }
                </Show>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(|| view! { <App /> });
}
