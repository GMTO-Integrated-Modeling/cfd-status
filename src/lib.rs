use std::{
    fmt::Display,
    io,
    num::{ParseFloatError, ParseIntError},
    ops::Mul,
    path::Path,
    process::{Command, Stdio},
    string::FromUtf8Error,
};

use chrono::{Duration, Local};
use regex::Regex;

pub const UPDATE_TIME: usize = 180;
const ROOT: &str = "/shared";
const RATE: usize = 20; //Hz

#[derive(Debug, Default, Clone)]
pub struct ElapsedPerStep {
    value: f64,
    sample: usize,
}

impl ElapsedPerStep {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn update(&mut self, value: f64) -> &mut Self {
        let n = self.sample as f64;
        self.sample += 1;
        self.value = (self.value * n + value) / self.sample as f64;
        self
    }
}

impl Display for ElapsedPerStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>8.2}", self.value)
    }
}

impl Mul<f64> for &ElapsedPerStep {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.value * rhs
    }
}

#[derive(Debug, Default, Clone)]
pub struct Case {
    name: String,
    duration: usize,
    log: String,
    step: Option<usize>,
    time: f64,
    elapsed_per_step: ElapsedPerStep,
}

#[derive(Debug, thiserror::Error)]
pub enum CaseError {
    #[error("failed to call shell")]
    Command(#[from] io::Error),
    #[error("failed to convert to UTF8")]
    UTF8(#[from] FromUtf8Error),
    #[error("failed to parse String")]
    ParseFloat(#[from] ParseFloatError),
    #[error("failed to parse String")]
    ParseInt(#[from] ParseIntError),
    #[error("No TimeStep/Time match found")]
    Regex,
    #[error("grep TimeStep failed")]
    Grep,
}

pub type Result<T> = std::result::Result<T, CaseError>;

impl Case {
    pub fn new<S: ToString>(name: S, duration: usize, log: S) -> Self {
        Self {
            name: name.to_string(),
            duration,
            log: log.to_string(),
            ..Default::default()
        }
    }
    pub fn log_file(&self) -> String {
        Path::new(ROOT)
            .join(&self.name)
            .join(&self.log)
            .to_str()
            .unwrap()
            .to_string()
    }
    pub fn update(&mut self) -> Result<&mut Self> {
        let pattern = Regex::new(r"TimeStep\s+(\d+): Time\s+(\d+\.\d+e[+-]?\d+)").unwrap();

        let graph = Command::new("grep")
            .arg("TimeStep")
            .arg(&self.log_file())
            .stdout(Stdio::piped())
            .spawn()?;
        let svg = Command::new("tail")
            .arg("-n1")
            .stdin(Stdio::from(graph.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn()?;
        let output = svg.wait_with_output()?;
        if output.status.success() {
            let time_step = String::from_utf8(output.stdout)?;

            // Match the pattern against the input string
            if let Some(captures) = pattern.captures(&time_step) {
                // Extract the captured groups
                let time_step = captures
                    .get(1)
                    .map_or("", |m| m.as_str())
                    .parse::<usize>()?;
                let time_value = captures.get(2).map_or("", |m| m.as_str()).parse::<f64>()?;

                let diff_step = time_step - self.step.unwrap_or_else(|| time_step);
                self.step = Some(time_step);
                self.time = time_value;
                if diff_step > 0 {
                    self.elapsed_per_step
                        .update(UPDATE_TIME as f64 / diff_step as f64);
                }
            } else {
                return Err(CaseError::Regex);
            }
        } else {
            return Err(CaseError::Grep);
        }

        Ok(self)
    }
    pub fn eta_secs(&self) -> i64 {
        let n_step = self.duration * RATE - self.step.unwrap();
        (&self.elapsed_per_step * n_step as f64) as i64
    }
}

impl Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eta = Local::now() + Duration::seconds(self.eta_secs());
        write!(
            f,
            "{:<20}{:>8}{:>10.2}{:}{:>20}",
            self.name,
            self.step.unwrap(),
            self.time,
            self.elapsed_per_step,
            eta.format("%Y-%m-%d %H:%M")
        )
    }
}
