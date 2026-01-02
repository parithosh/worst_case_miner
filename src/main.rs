use clap::Parser;
use log::info;
use std::process::Command;
use std::time::Instant;

mod account_miner;
mod storage_miner;

#[cfg(feature = "cuda")]
mod cuda_miner;

/// A mining program to create deep branches in ERC20 contract storage and account trie
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target depth for the storage/account branch
    #[arg(short, long)]
    depth: usize,

    /// Number of threads to use for mining (default: number of CPU cores)
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// Use CUDA acceleration if available
    #[arg(long)]
    cuda: bool,

    /// Deployer address for CREATE2 (hex string, default: 0x0000...)
    #[arg(long)]
    deployer: Option<String>,

    /// Number of contracts to deploy via CREATE2
    #[arg(long)]
    num_contracts: Option<usize>,

    /// Path to contract init code for CREATE2 hash calculation
    #[arg(long)]
    init_code: Option<String>,

    /// Output file for CREATE2 accounts JSON
    #[arg(long, default_value = "create2_accounts.json")]
    accounts_output: String,
}

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    info!("Starting mining for depth: {}", args.depth);

    #[cfg(feature = "cuda")]
    {
        if args.cuda && cuda_miner::cuda_available() {
            info!("Using CUDA acceleration");

            // Verify CUDA keccak matches CPU
            let test_addr: [u8; 20] = [
                0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22,
                0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
            ];
            let cpu_result = storage_miner::calculate_storage_slot(&test_addr, 0);
            let cuda_result = cuda_miner::verify_cuda_keccak(&test_addr, 0);

            if cpu_result == cuda_result {
                info!("CUDA keccak verification PASSED");

                // Also verify that CUDA PRNG generates valid addresses/keys
                let (prng_addr, prng_key) = cuda_miner::debug_cuda_prng(12345, 0);
                let cpu_key_for_prng_addr = storage_miner::calculate_storage_slot(&prng_addr, 0);
                if prng_key == cpu_key_for_prng_addr {
                    info!("CUDA PRNG verification PASSED");
                    info!("  Sample address: 0x{}", hex::encode(&prng_addr));
                    info!("  Sample key:     0x{}", hex::encode(&prng_key[..8]));
                } else {
                    log::error!("CUDA PRNG verification FAILED!");
                    log::error!("  Address: 0x{}", hex::encode(&prng_addr));
                    log::error!("  CUDA key: 0x{}", hex::encode(&prng_key));
                    log::error!("  CPU key:  0x{}", hex::encode(&cpu_key_for_prng_addr));
                }
            } else {
                log::error!("CUDA keccak verification FAILED!");
                log::error!("CPU:  0x{}", hex::encode(&cpu_result));
                log::error!("CUDA: 0x{}", hex::encode(&cuda_result));
                log::error!("Disabling CUDA, falling back to CPU");
            }
        } else if args.cuda {
            info!("CUDA requested but not available, falling back to CPU");
            info!("Using {} CPU threads", args.threads);
        } else {
            info!("Using {} CPU threads", args.threads);
        }
    }

    #[cfg(not(feature = "cuda"))]
    {
        if args.cuda {
            info!("CUDA support not compiled. Rebuild with --features cuda");
        }
        info!("Using {} CPU threads", args.threads);
    }

    // Mine CREATE2 accounts if requested
    if let Some(num_contracts) = args.num_contracts {
        // Parse deployer address
        let deployer = if let Some(deployer_str) = args.deployer {
            parse_address(&deployer_str).expect("Invalid deployer address")
        } else {
            [0u8; 20] // Default to zero address
        };

        // Load or generate init code
        let init_code = if let Some(init_code_path) = args.init_code {
            // Check if it's a .sol file or a hex file
            if init_code_path.ends_with(".sol") {
                // Compile the Solidity file to get bytecode
                info!("Compiling Solidity contract: {}", init_code_path);
                compile_solidity_to_bytecode(&init_code_path)
                    .expect("Failed to compile Solidity contract")
            } else if init_code_path.ends_with(".hex") || init_code_path.ends_with(".bin") {
                // Read hex bytecode from file
                info!("Loading bytecode from: {}", init_code_path);
                let hex_content = std::fs::read_to_string(&init_code_path)
                    .expect("Failed to read bytecode file");
                let hex_content = hex_content.trim();
                let hex_content = hex_content.strip_prefix("0x").unwrap_or(hex_content);
                hex::decode(hex_content).expect("Invalid hex in bytecode file")
            } else {
                // Assume it's raw bytecode
                std::fs::read(&init_code_path).expect("Failed to read init code file")
            }
        } else if args.depth > 0 {
            // No init code provided but depth specified - generate and compile a contract with the specified depth
            info!("No init code provided. Generating contract with depth {}...", args.depth);

            // First, mine storage slots for the contract
            let branch = storage_miner::mine_deep_branch(args.depth, args.threads, false);

            // Generate the contract
            storage_miner::generate_contract(&branch);

            // Compile the generated contract
            let contract_path = "contracts/WorstCaseERC20.sol";
            info!("Compiling generated contract: {}", contract_path);
            compile_solidity_to_bytecode(contract_path)
                .expect("Failed to compile generated contract")
        } else {
            panic!("For CREATE2 mining, either provide --init-code or specify --depth to auto-generate a contract");
        };

        account_miner::mine_create2_accounts(
            deployer,
            num_contracts,
            args.depth,
            args.threads,
            &init_code,
            &args.accounts_output,
        );

        // Exit after CREATE2 mining - don't continue to storage mining
        return;
    }

    let start_time = Instant::now();

    // Mine for the deep branch (storage)
    let branch = storage_miner::mine_deep_branch(args.depth, args.threads, args.cuda);

    let elapsed = start_time.elapsed();

    // Output results
    storage_miner::print_results(&branch, elapsed.as_secs_f64());

    // Generate contract with mined storage keys
    storage_miner::generate_contract(&branch);
}

/// Parse hex address string to bytes
fn compile_solidity_to_bytecode(sol_path: &str) -> Result<Vec<u8>, String> {
    // Run solc to compile the contract with consistent metadata settings
    let output = Command::new("solc")
        .args(&["--optimize", "--optimize-runs", "200", "--bin", "--metadata-hash", "none", sol_path])
        .output()
        .map_err(|e| format!("Failed to run solc: {}. Make sure solc is installed.", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Solidity compilation failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find the binary output (it comes after "Binary:" line)
    let lines: Vec<&str> = stdout.lines().collect();
    let mut found_binary = false;
    for line in lines {
        if found_binary {
            // This is the bytecode line
            let bytecode_hex = line.trim();
            if !bytecode_hex.is_empty() {
                return hex::decode(bytecode_hex)
                    .map_err(|e| format!("Failed to decode bytecode hex: {}", e));
            }
        }
        if line.contains("Binary:") {
            found_binary = true;
        }
    }

    Err("Could not find bytecode in solc output".to_string())
}

fn parse_address(hex_str: &str) -> Result<[u8; 20], String> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    if hex_str.len() != 40 {
        return Err(format!(
            "Address must be 40 hex characters, got {}",
            hex_str.len()
        ));
    }

    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {e}"))?;

    let mut address = [0u8; 20];
    address.copy_from_slice(&bytes);
    Ok(address)
}
