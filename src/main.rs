use std::env;
use std::process::ExitCode;
use std::fs;
use binwalk::Binwalk;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("사용법: {} <firmware.bin>", args[0]);
        return ExitCode::FAILURE;
    }
    let file_path = &args[1];

    eprintln!("=== [1] 기본 스캔 ===");
    if let Err(e) = run_basic_scan(file_path) {
        eprintln!("[오류] 기본 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    eprintln!("\n=== [2] Matryoshka (재귀적 분석) ===");
    if let Err(e) = run_matryoshka(file_path) {
        eprintln!("[오류] Matryoshka 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    eprintln!("\n=== [3] 추출 ===");
    if let Err(e) = run_extract(file_path) {
        eprintln!("[오류] 추출 실패: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn run_basic_scan(file_path: &str) -> Result<(), String> {
    let binwalker = match Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,
        None,
        None,
        false,
    ) {
        Ok(b) => b,
        Err(e) => return Err(e.message),
    };

    let file_data = match fs::read(&binwalker.base_target_file) {
        Ok(d) => d,
        Err(e) => return Err(e.to_string()),
    };
    
    let file_map = binwalker.scan(&file_data);
    
    for result in &file_map {
        println!("{:#X}  {}", result.offset, result.description);
    }
    
    Ok(())
}

fn run_matryoshka(file_path: &str) -> Result<(), String> {
    let binwalker = match Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,
        None,
        None,
        false,
    ) {
        Ok(b) => b,
        Err(e) => return Err(e.message),
    };

    let file_data = match fs::read(&binwalker.base_target_file) {
        Ok(d) => d,
        Err(e) => return Err(e.to_string()),
    };
    
    let file_map = binwalker.scan(&file_data);
    
    for result in &file_map {
        println!("{:#X}  {}", result.offset, result.description);
    }
    
    Ok(())
}

fn run_extract(file_path: &str) -> Result<(), String> {
    let binwalker = match Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,
        None,
        None,
        false,
    ) {
        Ok(b) => b,
        Err(e) => return Err(e.message),
    };

    let file_data = match fs::read(&binwalker.base_target_file) {
        Ok(d) => d,
        Err(e) => return Err(e.to_string()),
    };
    
    let file_map = binwalker.scan(&file_data);
    let extraction_results = binwalker.extract(&file_data, &binwalker.base_target_file, &file_map);
    
    for (id, result) in &extraction_results {
        if result.success {
            let size_str = match result.size {
                Some(s) => format!("{} 바이트", s),
                None => "크기 없음".to_string(),
            };
            println!("[성공] 추출됨 - 크기: {}, 출력: {} (ID: {})", 
                     size_str, result.output_directory, id);
        } else {
            let size_str = match result.size {
                Some(s) => format!("{} 바이트", s),
                None => "크기 없음".to_string(),
            };
            println!("[실패] 추출 실패 - 크기: {} (ID: {})", 
                     size_str, id);
        }
    }
    
    Ok(())
}
