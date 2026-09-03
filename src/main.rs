#![windows_subsystem = "console"]

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// ─── Cores ANSI (funcionam no Windows 10+ com VT100) ──────────────────────
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const RED: &str = "\x1b[91m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

fn main() {
    enable_ansi_support();
    print_banner();

    let stats = run_cleanup();
    print_summary(&stats);

    // ── Limpeza de cache dos navegadores ───────────────────────────────────
    clean_browsers();

    // ── Limpeza de credenciais ─────────────────────────────────────────────
    clean_credentials();

    pause();
}

// ═══════════════════════════════════════════════════════════════════════════
//  NAVEGADORES — Chrome, Edge, Firefox
// ═══════════════════════════════════════════════════════════════════════════

/// Estrutura que descreve um navegador e suas pastas de cache
struct Browser {
    name: &'static str,
    /// Processo a ser encerrado antes de limpar (nome do .exe)
    process: &'static str,
    /// Subpastas de cache relativas a %LOCALAPPDATA% ou %APPDATA%
    /// Cada entrada: (base: "local" | "roaming", caminho relativo)
    cache_dirs: &'static [(&'static str, &'static str)],
}

const BROWSERS: &[Browser] = &[
    Browser {
        name: "Google Chrome",
        process: "chrome.exe",
        cache_dirs: &[
            ("local", "Google\\Chrome\\User Data\\Default\\Cache"),
            ("local", "Google\\Chrome\\User Data\\Default\\Code Cache"),
            ("local", "Google\\Chrome\\User Data\\Default\\GPUCache"),
            ("local", "Google\\Chrome\\User Data\\Default\\Media Cache"),
            (
                "local",
                "Google\\Chrome\\User Data\\Default\\Service Worker\\CacheStorage",
            ),
            (
                "local",
                "Google\\Chrome\\User Data\\Default\\Service Worker\\ScriptCache",
            ),
            ("local", "Google\\Chrome\\User Data\\ShaderCache"),
            (
                "local",
                "Google\\Chrome\\User Data\\Default\\Network\\Cookies",
            ), // arquivo
        ],
    },
    Browser {
        name: "Microsoft Edge",
        process: "msedge.exe",
        cache_dirs: &[
            ("local", "Microsoft\\Edge\\User Data\\Default\\Cache"),
            ("local", "Microsoft\\Edge\\User Data\\Default\\Code Cache"),
            ("local", "Microsoft\\Edge\\User Data\\Default\\GPUCache"),
            ("local", "Microsoft\\Edge\\User Data\\Default\\Media Cache"),
            (
                "local",
                "Microsoft\\Edge\\User Data\\Default\\Service Worker\\CacheStorage",
            ),
            (
                "local",
                "Microsoft\\Edge\\User Data\\Default\\Service Worker\\ScriptCache",
            ),
            ("local", "Microsoft\\Edge\\User Data\\ShaderCache"),
        ],
    },
    Browser {
        name: "Mozilla Firefox",
        process: "firefox.exe",
        cache_dirs: &[
            // Firefox usa um perfil com nome aleatório; escaneamos a pasta profiles
            ("roaming", "Mozilla\\Firefox\\Profiles"), // tratado especialmente
        ],
    },
];

/// Ponto de entrada para limpeza de navegadores
fn clean_browsers() {
    println!("{BOLD}{CYAN}╔══════════════════════════════════════════════════════╗{RESET}");
    println!("{BOLD}{CYAN}║              LIMPEZA DE CACHE — NAVEGADORES          ║{RESET}");
    println!("{BOLD}{CYAN}╚══════════════════════════════════════════════════════╝{RESET}");
    println!();

    let local = get_localappdata();
    let roaming = get_appdata();

    for browser in BROWSERS {
        println!("{BOLD}{YELLOW}► {}{RESET}", browser.name);

        // Tenta encerrar o processo do navegador antes de limpar
        kill_process(browser.process);

        let mut total_deleted = 0u64;
        let mut total_bytes = 0u64;
        let mut total_errors = 0u64;

        if browser.name == "Mozilla Firefox" {
            // Firefox: escaneia todos os perfis em %APPDATA%\Mozilla\Firefox\Profiles\
            let profiles_root = format!("{}\\Mozilla\\Firefox\\Profiles", roaming);
            let (d, b, e) = clean_firefox_profiles(&profiles_root);
            total_deleted += d;
            total_bytes += b;
            total_errors += e;
        } else {
            for (base, rel_path) in browser.cache_dirs {
                let root = if *base == "local" { &local } else { &roaming };
                let full_path = format!("{}\\{}", root, rel_path);
                let path = Path::new(&full_path);

                if !path.exists() {
                    println!("  {DIM}[N/A]{RESET}  {}", rel_path);
                    continue;
                }

                // Se for um arquivo (ex: Cookies), remove direto
                if path.is_file() {
                    match fs::remove_file(path) {
                        Ok(_) => {
                            println!("  {GREEN}[DEL]{RESET} {}", rel_path);
                            total_deleted += 1;
                        }
                        Err(e) => {
                            println!("  {RED}[ERRO]{RESET} {} — {}", rel_path, e);
                            total_errors += 1;
                        }
                    }
                } else {
                    let (d, b, e) = delete_contents(path);
                    total_deleted += d;
                    total_bytes += b;
                    total_errors += e;
                    let mb = b as f64 / 1_048_576.0;
                    if d > 0 || e > 0 {
                        println!(
                            "  {GREEN}[OK]{RESET}  {} — {} item(s)  ({:.2} MB)  {} erro(s)",
                            rel_path, d, mb, e
                        );
                    } else {
                        println!("  {CYAN}[VAZIO]{RESET} {}", rel_path);
                    }
                }
            }
        }

        let mb = total_bytes as f64 / 1_048_576.0;
        println!(
            "  {BOLD}Total: {} item(s) removido(s)  {:.2} MB liberado(s)  {} erro(s){RESET}",
            total_deleted, mb, total_errors
        );
        println!();
    }
}

/// Limpa o cache de todos os perfis do Firefox encontrados
fn clean_firefox_profiles(profiles_root: &str) -> (u64, u64, u64) {
    let mut total = (0u64, 0u64, 0u64);

    let path = Path::new(profiles_root);
    if !path.exists() {
        println!("  {DIM}[N/A]{RESET}  Perfis do Firefox não encontrados");
        return total;
    }

    // Subpastas de cache dentro de cada perfil
    let cache_subdirs = [
        "cache2",
        "cache2\\entries",
        "OfflineCache",
        "startupCache",
        "thumbnails",
        "shader-cache",
        "storage\\default",
        "storage\\permanent\\chrome\\idb",
    ];

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return total,
    };

    for entry in entries.flatten() {
        let profile_path = entry.path();
        if !profile_path.is_dir() {
            continue;
        }

        let profile_name = profile_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        println!("  {DIM}Perfil: {}{RESET}", profile_name);

        for subdir in &cache_subdirs {
            let cache_path = profile_path.join(subdir);
            if !cache_path.exists() {
                continue;
            }

            let (d, b, e) = delete_contents(&cache_path);
            total.0 += d;
            total.1 += b;
            total.2 += e;

            if d > 0 {
                let mb = b as f64 / 1_048_576.0;
                println!(
                    "    {GREEN}[OK]{RESET}  {} — {} item(s)  ({:.2} MB)",
                    subdir, d, mb
                );
            }
        }
    }

    total
}

/// Encerra um processo pelo nome (equivalente a `taskkill /F /IM nome.exe`)
fn kill_process(process_name: &str) {
    let result = Command::new("taskkill")
        .args(["/F", "/IM", process_name])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            println!(
                "  {YELLOW}[ENCERRADO]{RESET} {} fechado para limpeza",
                process_name
            );
        }
        _ => {
            // Processo não estava aberto — ok, segue em frente silenciosamente
        }
    }
}

/// Retorna %LOCALAPPDATA% com fallback
fn get_localappdata() -> String {
    env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        let user = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
        format!("{}\\AppData\\Local", user)
    })
}

/// Retorna %APPDATA% com fallback
fn get_appdata() -> String {
    env::var("APPDATA").unwrap_or_else(|_| {
        let user = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
        format!("{}\\AppData\\Roaming", user)
    })
}

// ═══════════════════════════════════════════════════════════════════════════
//  CREDENCIAIS — cmdkey.exe
// ═══════════════════════════════════════════════════════════════════════════

/// Lista todas as credenciais via `cmdkey /list` e remove cada uma com
/// `cmdkey /delete:<target>`.  Funciona sem janela gráfica, 100% silencioso.
fn clean_credentials() {
    println!("{BOLD}{YELLOW}► Limpeza de Credenciais Armazenadas{RESET}");

    // Pergunta confirmação antes de deletar tudo
    print!(
        "  {YELLOW}[ATENÇÃO]{RESET} Isso removerá TODAS as credenciais salvas. Confirmar? (s/N): "
    );
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let confirmed = matches!(
        input.trim().to_lowercase().as_str(),
        "s" | "sim" | "y" | "yes"
    );

    if !confirmed {
        println!("  {DIM}[IGNORADO]{RESET} Limpeza de credenciais cancelada pelo usuário.");
        println!();
        return;
    }

    // Obtém a lista de credenciais
    let targets = list_credentials();

    if targets.is_empty() {
        println!("  {CYAN}[VAZIO]{RESET} Nenhuma credencial encontrada.");
        println!();
        return;
    }

    println!(
        "  {DIM}Encontradas {} credencial(is):{RESET}",
        targets.len()
    );

    let mut deleted = 0u32;
    let mut errors = 0u32;

    for target in &targets {
        let trimmed = target.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Remove: cmdkey /delete:<target>
        let result = Command::new("cmdkey")
            .arg(format!("/delete:{}", trimmed))
            .output();

        match result {
            Ok(out) if out.status.success() => {
                println!("  {GREEN}[DEL]{RESET} {}", trimmed);
                deleted += 1;
            }
            Ok(out) => {
                // cmdkey retorna stderr com detalhes
                let msg = String::from_utf8_lossy(&out.stderr);
                let msg = msg.trim();
                println!(
                    "  {RED}[ERRO]{RESET} {} — {}",
                    trimmed,
                    if msg.is_empty() {
                        "falha desconhecida"
                    } else {
                        msg
                    }
                );
                errors += 1;
            }
            Err(e) => {
                println!("  {RED}[ERRO]{RESET} {} — {}", trimmed, e);
                errors += 1;
            }
        }
    }

    println!();
    println!(
        "  {BOLD}Credenciais:{RESET} {} removida(s)  {} erro(s)",
        deleted, errors
    );
    println!();
}

/// Executa `cmdkey /list` e extrai os nomes (targets) de cada credencial.
/// O output do cmdkey tem o formato:
///
///   Destino atual:                        (ou "Target:" em inglês)
///     Credencial do Windows: DOMAIN\user
///     Usuário: ...
///
/// A linha com o target começa com espaços e contém um identificador após `:`.
fn list_credentials() -> Vec<String> {
    let output = Command::new("cmdkey").arg("/list").output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            println!(
                "  {RED}[ERRO]{RESET} Não foi possível executar cmdkey: {}",
                e
            );
            return vec![];
        }
    };

    // cmdkey imprime em UTF-16 no Windows; tentamos UTF-8 e depois lossy
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    parse_cmdkey_targets(&stdout)
}

/// Analisa a saída do `cmdkey /list` e retorna os targets.
///
/// Exemplo de saída (PT-BR):
///   Destino atual:
///     Credencial do Windows: Domain:target=MEUSERVIDOR
///     Tipo: Domínio
///     Usuário: usuario
///
/// Exemplo de saída (EN):
///   Currently stored credentials:
///     Target: LegacyGeneric:target=meu_app
///     Type: Generic
///     User: user@email.com
fn parse_cmdkey_targets(output: &str) -> Vec<String> {
    let mut targets = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        // Português: "Credencial do Windows: <target>" ou "Credencial Genérica: <target>"
        // Inglês:    "Target: <target>"
        // Ambos contêm o separador ": " e o target após ele
        let is_target_line = trimmed.to_lowercase().starts_with("destino:")
            || trimmed.to_lowercase().starts_with("target:")
            || trimmed.to_lowercase().starts_with("credencial do windows:")
            || trimmed.to_lowercase().starts_with("credencial genérica:")
            || trimmed
                .to_lowercase()
                .starts_with("credencial de certificado:")
            || trimmed.to_lowercase().starts_with("windows credential:")
            || trimmed.to_lowercase().starts_with("generic credential:")
            || trimmed
                .to_lowercase()
                .starts_with("certificate credential:");

        if is_target_line {
            // Pega tudo após o primeiro ": "
            if let Some(pos) = trimmed.find(": ") {
                let raw_target = trimmed[pos + 2..].trim();

                // Alguns targets têm prefixos como "Domain:target=X" ou "LegacyGeneric:target=X"
                // O cmdkey /delete aceita o nome completo como aparece na lista
                if !raw_target.is_empty() {
                    targets.push(raw_target.to_string());
                }
            }
        }
    }

    targets
}

// ═══════════════════════════════════════════════════════════════════════════
//  LIMPEZA DE ARQUIVOS TEMPORÁRIOS
// ═══════════════════════════════════════════════════════════════════════════

fn enable_ansi_support() {
    // No Windows 10+ o VT100 precisa ser habilitado via SetConsoleMode
    // Usamos um fallback simples via variável de ambiente
    #[cfg(target_os = "windows")]
    let _ = Command::new("cmd").args(["/c", ""]).output(); // força inicialização do subsistema de console
}

fn print_banner() {
    println!();
    println!("{BOLD}{CYAN}╔══════════════════════════════════════════════════════╗{RESET}");
    println!("{BOLD}{CYAN}║    LIMPEZA DE TEMPORÁRIOS, CACHE, NAVEGADORES        ║{RESET}");
    println!("{BOLD}{CYAN}╚══════════════════════════════════════════════════════╝{RESET}");
    println!();
}

struct CleanupStats {
    total_deleted: u64,
    total_bytes: u64,
    total_errors: u64,
}

impl CleanupStats {
    fn new() -> Self {
        Self {
            total_deleted: 0,
            total_bytes: 0,
            total_errors: 0,
        }
    }
}

fn run_cleanup() -> CleanupStats {
    let mut stats = CleanupStats::new();

    let temp_env = env::var("TEMP")
        .unwrap_or_else(|_| env::var("TMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string()));
    let userprofile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
    let windir = env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let appdata =
        env::var("APPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Roaming", userprofile));

    clean_dir_section(
        "Windows\\Temp\\TempHBCD",
        &format!("{}\\Temp\\TempHBCD", windir),
        &mut stats,
        true,
    );

    clean_dir_section(
        "Temporários do Sistema (%TEMP%)",
        &temp_env,
        &mut stats,
        false,
    );

    clean_dir_section(
        "Temporários do Usuário (AppData\\Local\\Temp)",
        &format!("{}\\AppData\\Local\\Temp", userprofile),
        &mut stats,
        false,
    );

    clean_dir_section(
        "Prefetch do Windows",
        &format!("{}\\Prefetch", windir),
        &mut stats,
        false,
    );

    clean_dir_section(
        "Temp do Windows",
        &format!("{}\\Temp", windir),
        &mut stats,
        false,
    );

    println!("{BOLD}{YELLOW}► Microsoft Teams{RESET}");
    for subdir in &[
        "tmp",
        "blob_storage",
        "Cache",
        "IndexedDB",
        "GPUCache",
        "databases",
    ] {
        let path = format!("{}\\Microsoft\\teams\\{}", appdata, subdir);
        clean_files_in_dir(&format!("  Teams\\{}", subdir), &path, &mut stats);
    }
    println!();

    stats
}

fn clean_dir_section(label: &str, path_str: &str, stats: &mut CleanupStats, remove_root: bool) {
    println!("{BOLD}{YELLOW}► {}{RESET}", label);
    let path = Path::new(path_str);

    if !path.exists() {
        println!("  {YELLOW}[IGNORADO]{RESET} Não encontrado: {}", path_str);
        println!();
        return;
    }

    if remove_root {
        match fs::remove_dir_all(path) {
            Ok(_) => {
                println!("  {GREEN}[OK]{RESET} Removido: {}", path_str);
                stats.total_deleted += 1;
            }
            Err(e) => {
                println!("  {RED}[ERRO]{RESET} {}: {}", path_str, e);
                stats.total_errors += 1;
            }
        }
    } else {
        clean_files_in_dir("  ", path_str, stats);
    }
    println!();
}

fn clean_files_in_dir(label: &str, path_str: &str, stats: &mut CleanupStats) {
    let path = Path::new(path_str);
    if !path.exists() {
        println!("  {YELLOW}[IGNORADO]{RESET} {}: não encontrado", label);
        return;
    }
    let (deleted, bytes, errors) = delete_contents(path);
    stats.total_deleted += deleted;
    stats.total_bytes += bytes;
    stats.total_errors += errors;

    let size_mb = bytes as f64 / 1_048_576.0;
    if deleted > 0 || errors > 0 {
        println!(
            "  {GREEN}[OK]{RESET} {}: {} item(s)  ({:.2} MB)  {} erro(s)",
            label, deleted, size_mb, errors
        );
    } else {
        println!("  {CYAN}[VAZIO]{RESET} {}: já estava limpo", label);
    }
}

fn delete_contents(dir: &Path) -> (u64, u64, u64) {
    let mut deleted = 0u64;
    let mut bytes = 0u64;
    let mut errors = 0u64;

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (0, 0, 1),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = fs::metadata(&path);

        if path.is_dir() {
            let (d, b, e) = delete_contents(&path);
            deleted += d;
            bytes += b;
            errors += e;
            let _ = fs::remove_dir(&path);
        } else {
            if let Ok(m) = meta {
                bytes += m.len();
            }
            force_remove(&path, &mut deleted, &mut errors);
        }
    }
    (deleted, bytes, errors)
}

fn force_remove(path: &PathBuf, deleted: &mut u64, errors: &mut u64) {
    if fs::remove_file(path).is_ok() {
        *deleted += 1;
        return;
    }
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        let _ = fs::set_permissions(path, perms);
    }
    match fs::remove_file(path) {
        Ok(_) => *deleted += 1,
        Err(_) => *errors += 1,
    }
}

fn print_summary(stats: &CleanupStats) {
    let size_mb = stats.total_bytes as f64 / 1_048_576.0;
    println!("{BOLD}{CYAN}╔══════════════════════════════════════════════════════╗{RESET}");
    println!("{BOLD}{CYAN}║                    RESUMO — ARQUIVOS                 ║{RESET}");
    println!("{BOLD}{CYAN}╠══════════════════════════════════════════════════════╣{RESET}");
    println!(
        "{BOLD}{CYAN}║{RESET}  Itens removidos : {GREEN}{:<35}{RESET}{BOLD}{CYAN}║{RESET}",
        stats.total_deleted
    );
    println!(
        "{BOLD}{CYAN}║{RESET}  Espaço liberado : {GREEN}{:<35}{RESET}{BOLD}{CYAN}║{RESET}",
        format!("{:.2} MB", size_mb)
    );
    println!(
        "{BOLD}{CYAN}║{RESET}  Erros           : {RED}{:<35}{RESET}{BOLD}{CYAN}║{RESET}",
        stats.total_errors
    );
    println!("{BOLD}{CYAN}╚══════════════════════════════════════════════════════╝{RESET}");
    println!();
}

fn pause() {
    print!("{BOLD}Pressione ENTER para sair...{RESET}");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}
