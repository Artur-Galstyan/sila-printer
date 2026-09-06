use std::{error::Error, process::Command};

const PRINTER_NAME: &str = "HP_Smart_Tank_7000_series__807313_";

#[derive(Debug, Clone, PartialEq)]
pub struct Printer {
    name: String,
    status: PrinterStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum PrinterStatus {
    Idle,
}

impl PrinterStatus {
    pub fn from_str(s: &str) -> PrinterStatus {
        match s {
            "idle" => PrinterStatus::Idle,
            _ => PrinterStatus::Idle,
        }
    }
}

pub fn list_printers() -> Result<Vec<Printer>, Box<dyn Error>> {
    let command_output = Command::new("lpstat").arg("-p").output()?;

    if !command_output.status.success() {
        return Err(Box::new(std::io::Error::other("lpstat failed")));
    }

    let output = String::from_utf8_lossy(&command_output.stdout);
    let lines: Vec<&str> = output.lines().collect();

    let printers: Vec<Printer> = lines
        .iter()
        .flat_map(|line| {
            // a line looks like this:
            // printer HP_Smart_Tank_7000_series__807313_ is idle.  enabled since Fri Sep  4 07:44:25 2026
            let parts: Vec<&str> = line.split_whitespace().collect();
            let first_part = parts.first()?;

            if *first_part != "printer" {
                return None;
            }

            let name = parts.get(1)?;

            // skip the "is" string
            let status = parts.get(3)?;

            // remove the trailing "." from the status
            let status = status.trim_end_matches('.');

            Some(Printer {
                name: name.to_string(),
                status: PrinterStatus::from_str(status),
            })
        })
        .collect();

    Ok(printers)
}
