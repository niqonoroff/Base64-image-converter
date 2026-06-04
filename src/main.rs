use base64::{engine::general_purpose::STANDARD, Engine as _};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::ui::{Color, ErrorMessageRenderConfig, RenderConfig, StyleSheet, Styled};
use inquire::{Select, Text};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

fn main() {
    setup_render_config();
    clear_screen();
    print_banner();

    loop {
        println!();
        let actions = vec![
            "🖼️   Картинка → Base64",
            "📜   Base64 → Картинка",
            "🚪   Выход",
        ];

        let action = match Select::new("➤  Выберите действие:", actions)
            .with_help_message("↑↓ перемещение, Enter выбор")
            .prompt()
        {
            Ok(a) => a,
            Err(_) => break,
        };

        match action {
            "🖼️   Картинка → Base64" => image_to_base64(),
            "📜   Base64 → Картинка" => base64_to_image(),
            "🚪   Выход" => {
                println!("\n{}\n", "👋  До свидания!".yellow().bold());
                break;
            }
            _ => break,
        }
    }
}

fn setup_render_config() {
    let mut render_config = RenderConfig::default();

    let lilac = Color::Rgb { r: 210, g: 180, b: 255 };
    let lilac_style = StyleSheet::new().with_fg(lilac);

    render_config.prompt = lilac_style;
    render_config.help_message = lilac_style;
    render_config.answer = StyleSheet::new().with_fg(Color::LightYellow);
    render_config.error_message = ErrorMessageRenderConfig::default_colored()
        .with_prefix(Styled::new("❌"))
        .with_message(StyleSheet::new().with_fg(Color::LightRed));

    render_config.highlighted_option_prefix = Styled::new("▶").with_fg(lilac);
    render_config.option = StyleSheet::new().with_fg(Color::White);
    render_config.selected_option = Some(StyleSheet::new().with_fg(lilac));

    inquire::set_global_render_config(render_config);
}

fn image_to_base64() {
    print_step("Картинка → Base64");

    let input_path = match ask_existing_file_path("📂  Путь к картинке") {
        Some(p) => p,
        None => return,
    };

    let data = match read_file_with_progress(&input_path) {
        Ok(d) => d,
        Err(e) => {
            print_error(&format!("Не удалось прочитать файл: {e}"));
            return;
        }
    };

    let file_size = data.len();
    println!("{}", format!("✔  Прочитано: {} байт", file_size).yellow());

    println!("{}", "⏳  Кодирование в Base64...".yellow());
    let b64 = STANDARD.encode(&data);
    let b64_size = b64.len();
    println!(
        "{}",
        format!("✔  Размер Base64-строки: {} байт", b64_size).yellow()
    );

    let default_out = input_path.with_extension("b64.txt");
    let suggested = default_out.to_string_lossy().to_string();
    let out_path_str = match Text::new("📝  Путь для сохранения .txt:")
        .with_default(&suggested)
        .with_help_message("Введите путь и нажмите Enter")
        .prompt()
    {
        Ok(p) => p,
        Err(_) => return,
    };
    let out_path = PathBuf::from(out_path_str);

    if let Err(e) = fs::write(&out_path, &b64) {
        print_error(&format!("Не удалось записать файл: {e}"));
        return;
    }

    println!(
        "{}",
        format!("✔  Файл сохранён: {}", out_path.display())
            .yellow()
            .bold()
    );
    println!(
        "{}",
        "💡  Теперь вы можете открыть этот файл в NQ-Editor и зашифровать его."
            .yellow()
    );

}

fn base64_to_image() {
    print_step("Base64 → Картинка");

    let path = match ask_existing_file_path("📂  Путь к TXT-файлу с Base64") {
        Some(p) => p,
        None => return,
    };

    let b64 = match fs::read_to_string(&path) {
        Ok(s) => s.trim().to_string(),
        Err(e) => {
            print_error(&format!("Не удалось прочитать файл: {e}"));
            return;
        }
    };

    if b64.is_empty() {
        print_error("Файл пустой. Отмена.");
        return;
    }

    println!("{}", "⏳  Декодирование Base64...".yellow());
    let data = match STANDARD.decode(&b64) {
        Ok(d) => d,
        Err(e) => {
            print_error(&format!("Некорректная Base64-строка: {e}"));
            return;
        }
    };
    println!("{}", format!("✔  Получено байт: {}", data.len()).yellow());

    let default_name = format!("restored_{}.bin", chrono_now_stub());
    let out_path_str = match Text::new("📝  Путь для сохранения картинки:")
        .with_default(&default_name)
        .with_help_message("Введите путь и нажмите Enter")
        .prompt()
    {
        Ok(p) => p,
        Err(_) => return,
    };
    let out_path = PathBuf::from(out_path_str);

    if let Err(e) = fs::write(&out_path, &data) {
        print_error(&format!("Не удалось записать файл: {e}"));
        return;
    }

    println!(
        "{}",
        format!("✔  Файл сохранён: {}", out_path.display())
            .yellow()
            .bold()
    );
    println!(
        "{}",
        "💡  Если расширение неверное — переименуйте вручную (например, в .png / .jpg)."
            .yellow()
    );

}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn print_banner() {
    let banner = r#"
   ╔══════════════════════════════════════════════════════╗
   ║                                                      ║
   ║               IMG ⇄ BASE64 CONVERTER                 ║
   ║                                                      ║
   ╚══════════════════════════════════════════════════════╝
"#;
    println!("{}", banner.yellow().bold());
}

fn print_step(title: &str) {
    println!("\n{}", format!("━━  {}  ━━", title).yellow().bold());
}

fn print_error(msg: &str) {
    println!("{} {}", "❌  Ошибка:".red().bold(), msg.red());
}

fn ask_existing_file_path(prompt: &str) -> Option<PathBuf> {
    loop {
        let path_str = match Text::new(&format!("{prompt}:"))
            .with_help_message("Введите путь и нажмите Enter")
            .prompt()
        {
            Ok(s) => s,
            Err(_) => return None,
        };
        let path_str = path_str.trim().trim_matches('"').trim_matches('\'');
        let p = PathBuf::from(path_str);
        if p.exists() && p.is_file() {
            return Some(p);
        }
        println!("{}", format!("⚠  Файл не найден: {}", p.display()).yellow());
    }
}

fn read_file_with_progress(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let total = file.metadata()?.len();
    let mut buf = Vec::with_capacity(total as usize);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.yellow/black}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("█▓▒░"),
    );
    pb.set_message("📖  Чтение");

    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        pb.inc(n as u64);
    }
    pb.finish_with_message("✔  Готово");
    Ok(buf)
}

fn chrono_now_stub() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}