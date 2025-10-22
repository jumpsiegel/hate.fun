use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

const PROGRAM_ID_FILE: &str = ".program-id";

/// Helper to load program ID from file
fn get_program_id() -> Pubkey {
    let program_id_str = std::fs::read_to_string(PROGRAM_ID_FILE)
        .expect("Failed to read .program-id file. Run ./scripts/deploy-native.sh first");
    Pubkey::from_str(program_id_str.trim()).expect("Invalid program ID")
}

/// Derive bucket PDA
fn derive_bucket_pda(program_id: &Pubkey, creator: &Pubkey, seed: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"bucket", creator.as_ref(), seed], program_id)
}

/// Derive main bucket PDA
fn derive_main_bucket_pda(program_id: &Pubkey, bucket: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"main", bucket.as_ref()], program_id)
}

/// Derive escrow A PDA
fn derive_escrow_a_pda(program_id: &Pubkey, bucket: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"escrow_a", bucket.as_ref()], program_id)
}

/// Derive escrow B PDA
fn derive_escrow_b_pda(program_id: &Pubkey, bucket: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"escrow_b", bucket.as_ref()], program_id)
}

/// Build create_bucket instruction
fn create_bucket_instruction(
    program_id: &Pubkey,
    payer: &Pubkey,
    bucket: &Pubkey,
    main_bucket: &Pubkey,
    escrow_a: &Pubkey,
    escrow_b: &Pubkey,
    address_a: &Pubkey,
    address_b: &Pubkey,
    creator_address: &Pubkey,
    creator_fee_bps: u16,
    claimer_fee_bps: u16,
    initial_last_swap: u64,
    min_increase_bps: u16,
    seed: &[u8; 32],
) -> Instruction {
    let mut data = vec![0u8]; // Discriminator 0
    data.extend_from_slice(address_a.as_ref());
    data.extend_from_slice(address_b.as_ref());
    data.extend_from_slice(creator_address.as_ref());
    data.extend_from_slice(&creator_fee_bps.to_le_bytes());
    data.extend_from_slice(&claimer_fee_bps.to_le_bytes());
    data.extend_from_slice(&initial_last_swap.to_le_bytes());
    data.extend_from_slice(&min_increase_bps.to_le_bytes());
    data.extend_from_slice(seed);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*bucket, false),
            AccountMeta::new(*main_bucket, false),
            AccountMeta::new(*escrow_a, false),
            AccountMeta::new(*escrow_b, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

/// Build deposit_to_escrow instruction
fn deposit_to_escrow_instruction(
    program_id: &Pubkey,
    depositor: &Pubkey,
    target_escrow: &Pubkey,
    amount: u64,
) -> Instruction {
    let mut data = vec![1u8]; // Discriminator 1
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*depositor, true),
            AccountMeta::new(*target_escrow, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

/// Build flush_escrow instruction
fn flush_escrow_instruction(
    program_id: &Pubkey,
    bucket: &Pubkey,
    main_bucket: &Pubkey,
    escrow_to_flush: &Pubkey,
) -> Instruction {
    let data = vec![2u8]; // Discriminator 2

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*bucket, false),
            AccountMeta::new(*main_bucket, false),
            AccountMeta::new(*escrow_to_flush, false),
        ],
        data,
    }
}

/// Build claim_payout instruction
fn claim_payout_instruction(
    program_id: &Pubkey,
    bucket: &Pubkey,
    main_bucket: &Pubkey,
    escrow_a: &Pubkey,
    escrow_b: &Pubkey,
    creator: &Pubkey,
    claimer: &Pubkey,
    winner: &Pubkey,
) -> Instruction {
    let data = vec![3u8]; // Discriminator 3

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*bucket, false),
            AccountMeta::new(*main_bucket, false),
            AccountMeta::new(*escrow_a, false),
            AccountMeta::new(*escrow_b, false),
            AccountMeta::new(*creator, false),
            AccountMeta::new(*claimer, true),
            AccountMeta::new(*winner, false),
        ],
        data,
    }
}

/// Build close_bucket instruction
fn close_bucket_instruction(
    program_id: &Pubkey,
    creator: &Pubkey,
    bucket: &Pubkey,
    main_bucket: &Pubkey,
    escrow_a: &Pubkey,
    escrow_b: &Pubkey,
) -> Instruction {
    let data = vec![4u8]; // Discriminator 4

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new(*bucket, false),
            AccountMeta::new(*main_bucket, false),
            AccountMeta::new(*escrow_a, false),
            AccountMeta::new(*escrow_b, false),
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_client() -> RpcClient {
        RpcClient::new_with_commitment(
            "http://localhost:8899".to_string(),
            CommitmentConfig::confirmed(),
        )
    }

    fn airdrop_if_needed(client: &RpcClient, pubkey: &Pubkey, amount: u64) {
        let balance = client.get_balance(pubkey).unwrap_or(0);
        if balance < amount {
            println!("Airdropping {} SOL to {}", amount as f64 / 1e9, pubkey);
            client
                .request_airdrop(pubkey, amount)
                .expect("Airdrop failed");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    #[test]
    #[ignore] // Run with: cargo test --test integration_client -- --ignored --nocapture
    fn test_create_bucket() {
        println!("\n=== Testing: Create Bucket ===\n");

        let client = setup_client();
        let program_id = get_program_id();
        let payer = Keypair::new();
        let address_a = Keypair::new().pubkey();
        let address_b = Keypair::new().pubkey();
        let creator_address = payer.pubkey();

        // Airdrop to payer
        airdrop_if_needed(&client, &payer.pubkey(), 5_000_000_000);

        // Generate seed
        let seed: [u8; 32] = rand::random();

        // Derive PDAs
        let (bucket, _) = derive_bucket_pda(&program_id, &creator_address, &seed);
        let (main_bucket, _) = derive_main_bucket_pda(&program_id, &bucket);
        let (escrow_a, _) = derive_escrow_a_pda(&program_id, &bucket);
        let (escrow_b, _) = derive_escrow_b_pda(&program_id, &bucket);

        println!("Bucket PDA: {}", bucket);
        println!("Main Bucket PDA: {}", main_bucket);
        println!("Escrow A PDA: {}", escrow_a);
        println!("Escrow B PDA: {}", escrow_b);

        // Build instruction
        let ix = create_bucket_instruction(
            &program_id,
            &payer.pubkey(),
            &bucket,
            &main_bucket,
            &escrow_a,
            &escrow_b,
            &address_a,
            &address_b,
            &creator_address,
            500,              // 5% creator fee
            50,               // 0.5% claimer fee
            1_000_000_000,    // 1 SOL initial swap
            500,              // 5% min increase
            &seed,
        );

        // Send transaction
        let recent_blockhash = client.get_latest_blockhash().expect("Failed to get blockhash");
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&tx).expect("Transaction failed");
        println!("\n✓ Bucket created successfully!");
        println!("Signature: {}", signature);

        // Verify accounts exist
        let bucket_account = client.get_account(&bucket).expect("Bucket account not found");
        println!("Bucket account size: {} bytes", bucket_account.data.len());
        assert_eq!(bucket_account.owner, program_id);
    }

    #[test]
    #[ignore]
    fn test_deposit_and_flush() {
        println!("\n=== Testing: Deposit and Flush ===\n");

        let client = setup_client();
        let program_id = get_program_id();
        let payer = Keypair::new();
        let address_a = Keypair::new().pubkey();
        let address_b = Keypair::new().pubkey();
        let depositor = Keypair::new();

        // Airdrop
        airdrop_if_needed(&client, &payer.pubkey(), 5_000_000_000);
        airdrop_if_needed(&client, &depositor.pubkey(), 5_000_000_000);

        // Generate seed
        let seed: [u8; 32] = rand::random();

        // Derive PDAs
        let (bucket, _) = derive_bucket_pda(&program_id, &payer.pubkey(), &seed);
        let (main_bucket, _) = derive_main_bucket_pda(&program_id, &bucket);
        let (escrow_a, _) = derive_escrow_a_pda(&program_id, &bucket);
        let (escrow_b, _) = derive_escrow_b_pda(&program_id, &bucket);

        // Create bucket first
        println!("Creating bucket...");
        let create_ix = create_bucket_instruction(
            &program_id,
            &payer.pubkey(),
            &bucket,
            &main_bucket,
            &escrow_a,
            &escrow_b,
            &address_a,
            &address_b,
            &payer.pubkey(),
            500,
            50,
            1_000_000_000, // 1 SOL initial
            500,           // 5% increase
            &seed,
        );

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        client.send_and_confirm_transaction(&tx).expect("Create failed");
        println!("✓ Bucket created");

        // Deposit to escrow B (1.1 SOL to exceed threshold of 1.05 SOL)
        println!("\nDepositing 1.1 SOL to escrow B...");
        let deposit_amount = 1_100_000_000; // 1.1 SOL
        let deposit_ix = deposit_to_escrow_instruction(
            &program_id,
            &depositor.pubkey(),
            &escrow_b,
            deposit_amount,
        );

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[deposit_ix],
            Some(&depositor.pubkey()),
            &[&depositor],
            recent_blockhash,
        );
        client.send_and_confirm_transaction(&tx).expect("Deposit failed");

        // Check escrow balance
        let escrow_balance = client.get_balance(&escrow_b).unwrap();
        println!("✓ Deposited. Escrow B balance: {} lamports", escrow_balance);
        assert!(escrow_balance >= deposit_amount);

        // Flush escrow B
        println!("\nFlushing escrow B...");
        let flush_ix = flush_escrow_instruction(&program_id, &bucket, &main_bucket, &escrow_b);

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[flush_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        client.send_and_confirm_transaction(&tx).expect("Flush failed");

        // Verify escrow B is now empty and main bucket has funds
        let escrow_b_balance = client.get_balance(&escrow_b).unwrap();
        let main_bucket_balance = client.get_balance(&main_bucket).unwrap();

        println!("✓ Flushed successfully!");
        println!("Escrow B balance after flush: {} lamports", escrow_b_balance);
        println!("Main bucket balance: {} lamports", main_bucket_balance);

        assert_eq!(escrow_b_balance, 0);
        assert!(main_bucket_balance >= deposit_amount);
    }

    #[test]
    #[ignore]
    fn test_close_bucket_before_flip() {
        println!("\n=== Testing: Close Bucket Before Flip ===\n");

        let client = setup_client();
        let program_id = get_program_id();
        let creator = Keypair::new();
        let address_a = Keypair::new().pubkey();
        let address_b = Keypair::new().pubkey();

        // Airdrop
        airdrop_if_needed(&client, &creator.pubkey(), 5_000_000_000);

        // Generate seed
        let seed: [u8; 32] = rand::random();

        // Derive PDAs
        let (bucket, _) = derive_bucket_pda(&program_id, &creator.pubkey(), &seed);
        let (main_bucket, _) = derive_main_bucket_pda(&program_id, &bucket);
        let (escrow_a, _) = derive_escrow_a_pda(&program_id, &bucket);
        let (escrow_b, _) = derive_escrow_b_pda(&program_id, &bucket);

        // Create bucket
        println!("Creating bucket...");
        let create_ix = create_bucket_instruction(
            &program_id,
            &creator.pubkey(),
            &bucket,
            &main_bucket,
            &escrow_a,
            &escrow_b,
            &address_a,
            &address_b,
            &creator.pubkey(),
            500,
            50,
            1_000_000_000,
            500,
            &seed,
        );

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&creator.pubkey()),
            &[&creator],
            recent_blockhash,
        );
        client.send_and_confirm_transaction(&tx).expect("Create failed");
        println!("✓ Bucket created");

        // Get creator balance before close
        let balance_before = client.get_balance(&creator.pubkey()).unwrap();
        println!("\nCreator balance before close: {} lamports", balance_before);

        // Close bucket immediately (no flips yet)
        println!("\nClosing bucket...");
        let close_ix = close_bucket_instruction(
            &program_id,
            &creator.pubkey(),
            &bucket,
            &main_bucket,
            &escrow_a,
            &escrow_b,
        );

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[close_ix],
            Some(&creator.pubkey()),
            &[&creator],
            recent_blockhash,
        );
        client.send_and_confirm_transaction(&tx).expect("Close failed");

        // Verify bucket account no longer exists
        let bucket_result = client.get_account(&bucket);
        assert!(bucket_result.is_err(), "Bucket should be closed");

        let balance_after = client.get_balance(&creator.pubkey()).unwrap();
        println!("✓ Bucket closed successfully!");
        println!("Creator balance after close: {} lamports", balance_after);
        println!("Recovered: {} lamports", balance_after.saturating_sub(balance_before));
    }

    #[test]
    #[ignore]
    fn test_full_flow() {
        println!("\n=== Testing: Full Flow (Create → Deposit → Flush → Multiple Flips) ===\n");

        let client = setup_client();
        let program_id = get_program_id();
        let creator = Keypair::new();
        let address_a = Keypair::new().pubkey();
        let address_b = Keypair::new().pubkey();
        let supporter_a = Keypair::new();
        let supporter_b = Keypair::new();

        // Airdrop
        airdrop_if_needed(&client, &creator.pubkey(), 5_000_000_000);
        airdrop_if_needed(&client, &supporter_a.pubkey(), 5_000_000_000);
        airdrop_if_needed(&client, &supporter_b.pubkey(), 5_000_000_000);

        let seed: [u8; 32] = rand::random();
        let (bucket, _) = derive_bucket_pda(&program_id, &creator.pubkey(), &seed);
        let (main_bucket, _) = derive_main_bucket_pda(&program_id, &bucket);
        let (escrow_a, _) = derive_escrow_a_pda(&program_id, &bucket);
        let (escrow_b, _) = derive_escrow_b_pda(&program_id, &bucket);

        // Step 1: Create bucket
        println!("Step 1: Creating bucket (initial 1 SOL, target: A)");
        let create_ix = create_bucket_instruction(
            &program_id, &creator.pubkey(), &bucket, &main_bucket, &escrow_a, &escrow_b,
            &address_a, &address_b, &creator.pubkey(), 500, 50, 1_000_000_000, 500, &seed,
        );
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[create_ix], Some(&creator.pubkey()), &[&creator], recent_blockhash);
        client.send_and_confirm_transaction(&tx).unwrap();
        println!("✓ Bucket created\n");

        // Step 2: B supporter deposits 1.1 SOL and flips to B
        println!("Step 2: B supporter deposits 1.1 SOL to flip control to B");
        let deposit_ix = deposit_to_escrow_instruction(&program_id, &supporter_b.pubkey(), &escrow_b, 1_100_000_000);
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[deposit_ix], Some(&supporter_b.pubkey()), &[&supporter_b], recent_blockhash);
        client.send_and_confirm_transaction(&tx).unwrap();

        let flush_ix = flush_escrow_instruction(&program_id, &bucket, &main_bucket, &escrow_b);
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[flush_ix], Some(&creator.pubkey()), &[&creator], recent_blockhash);
        client.send_and_confirm_transaction(&tx).unwrap();

        let pot = client.get_balance(&main_bucket).unwrap();
        println!("✓ Control flipped to B. Pot: {} SOL\n", pot as f64 / 1e9);

        // Step 3: A supporter counter-flips
        println!("Step 3: A supporter deposits 1.2 SOL to flip back to A");
        let deposit_ix = deposit_to_escrow_instruction(&program_id, &supporter_a.pubkey(), &escrow_a, 1_200_000_000);
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[deposit_ix], Some(&supporter_a.pubkey()), &[&supporter_a], recent_blockhash);
        client.send_and_confirm_transaction(&tx).unwrap();

        let flush_ix = flush_escrow_instruction(&program_id, &bucket, &main_bucket, &escrow_a);
        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[flush_ix], Some(&creator.pubkey()), &[&creator], recent_blockhash);
        client.send_and_confirm_transaction(&tx).unwrap();

        let pot = client.get_balance(&main_bucket).unwrap();
        println!("✓ Control flipped back to A. Pot: {} SOL\n", pot as f64 / 1e9);

        println!("=== Full flow test completed successfully! ===");
        println!("Final pot size: {} SOL", pot as f64 / 1e9);
    }

    #[test]
    #[ignore]
    fn test_validation_fees_too_high() {
        println!("\n=== Testing: Validation - Fees Too High ===\n");

        let client = setup_client();
        let program_id = get_program_id();
        let payer = Keypair::new();

        airdrop_if_needed(&client, &payer.pubkey(), 5_000_000_000);

        let seed: [u8; 32] = rand::random();
        let (bucket, _) = derive_bucket_pda(&program_id, &payer.pubkey(), &seed);
        let (main_bucket, _) = derive_main_bucket_pda(&program_id, &bucket);
        let (escrow_a, _) = derive_escrow_a_pda(&program_id, &bucket);
        let (escrow_b, _) = derive_escrow_b_pda(&program_id, &bucket);

        // Try to create with 21% total fees (should fail)
        let ix = create_bucket_instruction(
            &program_id, &payer.pubkey(), &bucket, &main_bucket, &escrow_a, &escrow_b,
            &Keypair::new().pubkey(), &Keypair::new().pubkey(), &payer.pubkey(),
            1600, // 16%
            500,  // 5% = 21% total
            1_000_000_000, 500, &seed,
        );

        let recent_blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], recent_blockhash);
        let result = client.send_and_confirm_transaction(&tx);

        assert!(result.is_err(), "Transaction should fail with fees > 20%");
        println!("✓ Correctly rejected fees > 20%");
    }
}
