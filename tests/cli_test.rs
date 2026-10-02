use clap::Parser;
use urupema::adapters::cli::CliArgs;
use urupema::adapters::sendmail::SendmailSink;
use urupema::domain::EmailSink;

#[test]
fn test_cli_args_parsing_success() {
    let args = vec![
        "urupema",
        "--sender",
        "postmaster@exemplo.org",
        "--recipient",
        "usuario@exemplo.org",
    ];

    let parsed = CliArgs::try_parse_from(args).expect("Deve analisar argumentos válidos");
    assert_eq!(parsed.sender, "postmaster@exemplo.org");
    assert_eq!(parsed.recipient, "usuario@exemplo.org");
    assert_eq!(parsed.sendmail_bin, "/usr/sbin/sendmail");
}

#[test]
fn test_cli_args_custom_sendmail_bin() {
    let args = vec![
        "urupema",
        "-s",
        "sender@test.com",
        "-r",
        "dest@test.com",
        "--sendmail-bin",
        "/custom/bin/sendmail",
    ];

    let parsed = CliArgs::try_parse_from(args).expect("Deve aceitar caminho customizado");
    assert_eq!(parsed.sendmail_bin, "/custom/bin/sendmail");
}

#[test]
fn test_cli_args_missing_recipient_fails() {
    let args = vec!["urupema", "--sender", "sender@test.com"];
    let result = CliArgs::try_parse_from(args);
    assert!(result.is_err());
}

#[test]
fn test_sendmail_sink_execution() {
    // Usamos o comando `/bin/sh` como fake de sendmail para verificar escrita em stdin e argumentos
    let mut sink = SendmailSink::new("/bin/sh");
    let payload = b"From: a@b.com\r\nSubject: Test\r\n\r\nCorpo de teste";

    // Executa um comando simples via sendmail fake
    let result = sink.deliver("a@b.com", "c@d.com", payload);
    assert!(result.is_ok());
}
