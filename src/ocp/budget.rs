#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitPolicyMode {
    Normal,
    ForbidCommit,
    ShadowCommitLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardMode {
    Return,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecConfig {
    pub step_cap: u32,
    pub commit_policy: CommitPolicyMode,
    pub guard_mode: GuardMode,
    pub enable_exec_cache: bool,
}

impl Default for ExecConfig {
    fn default() -> Self {
        Self {
            step_cap: 10_000,
            commit_policy: CommitPolicyMode::Normal,
            guard_mode: GuardMode::Return,
            enable_exec_cache: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetMeter {
    step_cap: u32,
    steps: u32,
}

impl BudgetMeter {
    pub fn new(config: ExecConfig) -> Self {
        Self {
            step_cap: config.step_cap,
            steps: 0,
        }
    }

    pub fn tick(&mut self) -> Result<(), BudgetError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.step_cap {
            return Err(BudgetError);
        }
        Ok(())
    }

    pub const fn steps(&self) -> u32 {
        self.steps
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetError;
