use cfd_status::{Case, CaseError, UPDATE_TIME};
use chrono::Local;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let mut cases = vec![
        Case::new("zen30az045_OS2", 1_200, "solve-672_14.out"),
        Case::new("zen30az090_OS2", 1_200, "solve-672_16.out"),
        Case::new("zen30az045_OS7", 900, "solve-672_15.out"),
        Case::new("zen30az090_OS7", 900, "solve-672_17.out"),
        Case::new("zen30az135_OS7", 900, "solve-672_18.out"),
        Case::new("zen30az045_CD12", 900, "solve-672_19.out"),
        Case::new("zen30az090_CD12", 900, "solve-672_20.out"),
        Case::new("zen30az180_CD12", 900, "solve-672_21.out"),
    ];

    let mut stdout = stdout();
    let error: Result<(), CaseError> = loop {
        let status = match cases
            .iter_mut()
            .map(|case| case.update().map(|case| case.to_string()))
            .collect::<Result<Vec<String>, CaseError>>()
        {
            Ok(status) => status,
            Err(e) => break Err(e),
        };

        std::process::Command::new("clear").status().unwrap();
        println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!(
            "{:20}{:>8}{:>10}{:>8}{:>20}",
            "Case", "%", "P.[s]", "I.[s]", "ETA"
        );
        print!("\r{:}", status.join("\n"));
        stdout.flush().unwrap();

        sleep(Duration::from_secs(UPDATE_TIME as u64));
    };
    Ok(error?)
}
