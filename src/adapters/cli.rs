use clap::Parser;

/// Argumentos de linha de comando para o filtro Postfix Urupema.
#[derive(Debug, Clone, Parser)]
#[command(name = "urupema")]
#[command(about = "Filtro Post-Queue MIME e sanitizador de e-mail para Postfix")]
pub struct CliArgs {
    /// Endereço de envelope do remetente original
    #[arg(short = 's', long = "sender")]
    pub sender: String,

    /// Endereço de envelope do destinatário original
    #[arg(short = 'r', long = "recipient")]
    pub recipient: String,

    /// Caminho para o executável sendmail do sistema
    #[arg(long = "sendmail-bin", default_value = "/usr/sbin/sendmail")]
    pub sendmail_bin: String,
}
