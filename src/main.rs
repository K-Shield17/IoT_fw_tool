use std::env;
use std::process::ExitCode;
use std::fs;
use binwalk::Binwalk;

fn main() -> ExitCode {
    // 인자 파싱: 오직 펌웨어 파일 경로 하나만 받음
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("사용법: {} <firmware.bin>", args[0]);
        return ExitCode::FAILURE;
    }
    let file_path = &args[1];

    // 1. 기본 스캔
    eprintln!("=== [1] 기본 스캔 ===");
    if let Err(e) = run_basic_scan(file_path) {
        eprintln!("[오류] 기본 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    // 2. -M 로직 (Matryoshka - 재귀적 분석)
    eprintln!("\n=== [2] Matryoshka (재귀적 분석) ===");
    if let Err(e) = run_matryoshka(file_path) {
        eprintln!("[오류] Matryoshka 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    // 3. -e 로직 (추출)
    eprintln!("\n=== [3] 추출 ===");
    if let Err(e) = run_extract(file_path) {
        eprintln!("[오류] 추출 실패: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// 기본 스캔 (옵션 없음)
fn run_basic_scan(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binwalker = Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,  // include filters
        None,  // exclude filters
        None,  // custom signatures
        false, // full_search
    )?;

    // 파일 데이터 읽기
    let file_data = fs::read(&binwalker.base_target_file)?;
    
    // 스캔 실행
    let file_map = binwalker.scan(&file_data);
    
    // 결과 출력
    for result in &file_map {
        println!("{:#X}  {}", result.offset, result.description);
    }
    
    Ok(())
}

/// -M 로직 (Matryoshka - 재귀적 분석)
fn run_matryoshka(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binwalker = Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,
        None,
        None,
        false,
    )?;

    let file_data = fs::read(&binwalker.base_target_file)?;
    let file_map = binwalker.scan(&file_data);
    
    for result in &file_map {
        println!("{:#X}  {}", result.offset, result.description);
    }
    
    Ok(())
}

/// -e 로직 (추출)
fn run_extract(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let binwalker = Binwalk::configure(
        Some(file_path.to_string()),
        Some("extractions".to_string()),
        None,
        None,
        None,
        false,
    )?;

    let file_data = fs::read(&binwalker.base_target_file)?;
    let file_map = binwalker.scan(&file_data);
    let extraction_results = binwalker.extract(&file_data, &binwalker.base_target_file, &file_map);
    
    // 추출 결과 출력
    for (id, result) in &extraction_results {
        if result.success {
            println!("[성공] {} 추출됨 (ID: {})", result.file_path, id);
        } else {
            println!("[실패] {} 추출 실패 (ID: {})", result.file_path, id);
        }
    }
    
    Ok(())
}
