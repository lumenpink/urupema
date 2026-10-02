use std::io::{self, Read};
use std::process::ExitCode;

use clap::Parser;
use urupema::adapters::{CliArgs, SendmailSink};
use urupema::application::Pipeline;
use urupema::domain::{ProcessResult, RawEmail};

// Códigos canônicos de saída sysexits.h utilizados por MTAs Unix / Postfix
const EX_USAGE: u8 = 64;
const EX_TEMPFAIL: u8 = 75;

fn main() -> ExitCode {
    let args = match CliArgs::try_parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(EX_USAGE);
        }
    };

    // Lê o payload RFC 5322 bruto da entrada padrão (stdin)
    let mut stdin_buf = Vec::new();
    if let Err(err) = io::stdin().read_to_end(&mut stdin_buf) {
        eprintln!("urupema: erro ao ler stdin: {err}");
        return ExitCode::from(EX_TEMPFAIL);
    }

    let raw = RawEmail::new(stdin_buf);
    let mut sink = SendmailSink::new(&args.sendmail_bin);

    match Pipeline::process(&raw, &args.sender, &args.recipient, &mut sink) {
        Ok((_, result)) => {
            match result {
                ProcessResult::Reinject => {
                    eprintln!("urupema: mensagem higienizada e reinjetada com sucesso");
                }
                ProcessResult::Bypass => {
                    eprintln!("urupema: mensagem em bypass entregue diretamente ao sink");
                }
                ProcessResult::Discard => {
                    eprintln!("urupema: mensagem descartada por politica");
                }
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("urupema: erro no pipeline: {err}");
            ExitCode::from(EX_TEMPFAIL)
        }
    }
}
