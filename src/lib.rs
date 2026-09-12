use std::process::Command;

const PRINTER_NAME: &str = "HP_Smart_Tank_7000_series__807313_";

#[derive(Debug, Clone, PartialEq)]
pub struct Printer {
    pub name: String,
    pub status: PrinterStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrinterStatus {
    Idle,
    Processing,
    Stopped,
    Unknown,
}

impl std::fmt::Display for PrinterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let printer_status = match self {
            PrinterStatus::Idle => "idle",
            PrinterStatus::Processing => "processing",
            PrinterStatus::Stopped => "stopped",
            PrinterStatus::Unknown => "unknown",
        };
        write!(f, "{}", printer_status)
    }
}

impl PrinterStatus {
    pub fn parse(s: &str) -> PrinterStatus {
        match s {
            "idle" => PrinterStatus::Idle,
            "processing" => PrinterStatus::Processing,
            "stopped" => PrinterStatus::Stopped,
            _ => PrinterStatus::Unknown,
        }
    }
}

pub fn get_printer() -> std::io::Result<Printer> {
    let command_output = Command::new("lpstat").arg("-p").output()?;

    if !command_output.status.success() {
        return Err(std::io::Error::other("lpstat failed"));
    }

    let output = String::from_utf8_lossy(&command_output.stdout);
    let lines: Vec<&str> = output.lines().collect();

    let printer = lines
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
                status: PrinterStatus::parse(status),
            })
        })
        .find(|p| p.name == PRINTER_NAME);

    printer.ok_or(std::io::Error::other("printer not found"))
}
