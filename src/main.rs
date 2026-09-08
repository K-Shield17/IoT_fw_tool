use std::env;
use std::process::ExitCode;
use std::fs;
use std::collections::{HashSet, VecDeque};
use binwalk::Binwalk;
use binwalk::AnalysisResults;
use binwalk::extractors;

const OUTPUT_DIRECTORY: &str = "extractions";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("사용법: {} <firmware.bin>", args[0]);
        return ExitCode::FAILURE;
    }
    let file_path = &args[1];

    // Binwalk 인스턴스 생성 (한 번만)
    let binwalker = match Binwalk::configure(
        Some(file_path.to_string()),
        Some(OUTPUT_DIRECTORY.to_string()),
        None,
        None,
        None,
        false,
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[오류] Binwalk 초기화 실패: {}", e.message);
            return ExitCode::FAILURE;
        }
    };

    eprintln!("=== Binwalk 자동 분석: 기본 스캔 + -Me ===");
    eprintln!("입력: {}", binwalker.base_target_file);
    eprintln!("출력: {}", binwalker.base_output_directory);

    // 재귀 처리용 큐
    let mut targets: VecDeque<String> = VecDeque::new();
    let mut processed: HashSet<String> = HashSet::new();

    // 초기 파일 추가
    targets.push_back(binwalker.base_target_file.clone());

    // 큐가 빌 때까지 반복
    while let Some(target_file) = targets.pop_front() {
        // 중복 처리 방지
        let canonical_key = match std::fs::canonicalize(&target_file) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => target_file.clone(),
        };

        if !processed.insert(canonical_key) {
            continue;
        }

        eprintln!("\n=== 분석: {} ===", target_file);

        // 파일 데이터 읽기
        let file_data = match fs::read(&target_file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[경고] 읽기 실패: {}: {}", target_file, e);
                continue;
            }
        };

        // 분석 + 추출 (true = 추출 활성화)
        let results: AnalysisResults = binwalker.analyze_buf(&file_data, &target_file, true);

        // 결과 출력
        if results.file_map.is_empty() {
            eprintln!("[정보] 시그니처 없음");
        } else {
            println!("{:<12} {:<12} {:<20} Description", "Decimal", "Hex", "Type");
            println!("{}", "-".repeat(85));

            for result in &results.file_map {
                println!("{:<12} 0x{:<10X} {:<20} {}", 
                         result.offset, result.offset, result.name, result.description);
            }
        }

        // 추출 결과 처리
        for (signature_id, extraction_result) in &results.extractions {
            if !extraction_result.success {
                eprintln!("[추출 실패] ID={} / 출력={}", 
                          signature_id, extraction_result.output_directory);
                continue;
            }

            eprintln!("[추출 성공] ID={} / 출력={}", 
                      signature_id, extraction_result.output_directory);

            // 재귀: do_not_recurse 가 false 이면 추출된 파일들을 큐에 추가
            if extraction_result.do_not_recurse {
                continue;
            }

            for extracted_file in extractors::common::get_extracted_files(&extraction_result.output_directory) {
                if extracted_file != target_file {
                    eprintln!("  → 재귀 대상 추가: {}", extracted_file);
                    targets.push_back(extracted_file);
                }
            }
        }
    }

    eprintln!("\n=== 분석 완료 ===");
    eprintln!("결과 디렉터리: {}", binwalker.base_output_directory);

    ExitCode::SUCCESS
}
