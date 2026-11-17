// Process-wide static counters (DESIGN.md section 3.6, context.py).
// The CLI keeps one RunContext per run and passes it to every render().

pub struct RunContext {
    pub player_id: i64,
    pub message_id: i64,
    pub mynum: i64,
    pub err: String,
}

pub fn run_context_fresh() -> RunContext {
    return RunContext {
        player_id: 1000000,
        message_id: 1,
        mynum: 0,
        err: "".to_string(),
    };
}
