use {
    borsh::BorshDeserialize,
    chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc},
    clap::{
        crate_description, crate_name, crate_version, value_t_or_exit, App, AppSettings, Arg,
        SubCommand,
    },
    nobilitydao::state::{
        HouseData, TitleData, MAX_KIND, MAX_RANK, MAX_VASSALS, MIN_KIND, MIN_RANK,
    },
    solana_clap_utils::{
        input_parsers::{keypair_of, pubkey_of, value_of},
        input_validators::{
            is_keypair, is_url, is_valid_percentage, is_valid_pubkey, is_within_range,
        },
    },
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        clock::UnixTimestamp,
        commitment_config::CommitmentConfig,
        native_token::lamports_to_sol,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    },
    std::{
        collections::HashMap,
        fmt::Debug,
        fmt::Display,
        fs::File,
        io::Write,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
};

struct Config {
    keypair: Keypair,
    json_rpc_url: String,
    verbose: bool,
}

pub fn is_short<T>(string: T) -> Result<(), String>
where
    T: AsRef<str> + Display,
{
    if string.as_ref().len() >= 128 {
        return Err(format!("too long: {}", string));
    }
    Ok(())
}

pub fn is_short_url<T>(string: T) -> Result<(), String>
where
    T: AsRef<str> + Display,
{
    if string.as_ref().len() == 0 {
        return Ok(());
    }
    // inlining is_url
    match url::Url::parse(string.as_ref()) {
        Ok(url) => {
            if url.has_host() {
            } else {
                return Err("no host provided".to_string());
            }
        }
        Err(err) => {
            return Err(format!("{}", err));
        }
    }
    is_short(string)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair [default: client keypair]"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .subcommand(
            SubCommand::with_name("show-house")
                .about("Display information about the given wallet's house")
                .arg(
                    Arg::with_name("user_address")
                        .value_name("USER_ADDRESS")
                        .alias("keypair")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .help("The address of the wallet whose house should be shown"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-house")
                .about("Create a house for the given user wallet")
                .arg(
                    Arg::with_name("user_address")
                        .value_name("USER_ADDRESS")
                        .alias("keypair")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .help("The address of the wallet whose house should be created"),
                )
                .arg(
                    Arg::with_name("coat_of_arms")
                        .long("coat-of-arms")
                        .value_name("COAT_OF_ARMS")
                        .takes_value(true)
                        .validator(is_short_url)
                        .default_value("")
                        .help("Coat of arms of the new house"),
                )
                .arg(
                    Arg::with_name("display_name")
                        .long("display-name")
                        .value_name("DISPLAY_NAME")
                        .takes_value(true)
                        .validator(is_short)
                        .help("Display name for the house"),
                ),
        )
        .subcommand(
            SubCommand::with_name("show-root-title")
                .about("Display information about the root title"),
        )
        .subcommand(
            SubCommand::with_name("show-title")
                .about("Display information about the given title")
                .arg(
                    Arg::with_name("title_address")
                        .value_name("TITLE_ADDRESS")
                        .alias("keypair")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .help("The address of the title that should be shown"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-title")
                .about("Create a title")
                .arg(
                    Arg::with_name("user_address")
                        .long("user-address")
                        .value_name("USER_ADDRESS")
                        .alias("keypair")
                        .validator(is_valid_pubkey)
                        .help("The address of the wallet which will hold the new title"),
                )
                .arg(
                    Arg::with_name("liege_address")
                        .long("liege-address")
                        .value_name("LIEGE_ADDRESS")
                        .validator(|s| {
                            if s.len() == 0 {
                                Ok(())
                            } else {
                                is_valid_pubkey(s)
                            }
                        })
                        .help("The address of the liege title - may be empty for root titles"),
                )
                .arg(
                    Arg::with_name("coat_of_arms")
                        .long("coat-of-arms")
                        .value_name("COAT_OF_ARMS")
                        .takes_value(true)
                        .validator(is_short_url)
                        .default_value("")
                        .help("Coat of arms of the new title"),
                )
                .arg(
                    Arg::with_name("display_name")
                        .long("display-name")
                        .value_name("DISPLAY_NAME")
                        .takes_value(true)
                        .validator(is_short)
                        .help("Display name for the title"),
                )
                .arg(
                    Arg::with_name("rank")
                        .long("rank")
                        .value_name("RANK")
                        .takes_value(true)
                        .validator(|s| is_within_range(s, MIN_RANK as usize, MAX_RANK as usize))
                        .help("Title rank"),
                )
                .arg(
                    Arg::with_name("kind")
                        .long("kind")
                        .value_name("KIND")
                        .takes_value(true)
                        .validator(|s| is_within_range(s, MIN_KIND as usize, MAX_KIND as usize))
                        .help("Title kind"),
                )
                .arg(
                    Arg::with_name("required_stake_lamports")
                        .long("required-stake-lamports")
                        .value_name("REQUIRED_STAKE_LAMPORTS")
                        .takes_value(true)
                        .help("Required number of lamports to stake when activating this title"),
                )
                .arg(
                    Arg::with_name("liege_vassal_index")
                        .long("liege-vassal-index")
                        .value_name("LIEGE_VASSAL_INDEX")
                        .takes_value(true)
                        .validator(|s| is_within_range(s, 0, MAX_VASSALS as usize))
                        .help("Index into the liege's vassal vector"),
                ),
        )
        .get_matches();

    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        Config {
            json_rpc_url: matches
                .value_of("json_rpc_url")
                .unwrap_or(&cli_config.json_rpc_url)
                .to_string(),
            keypair: read_keypair_file(
                matches
                    .value_of("keypair")
                    .unwrap_or(&cli_config.keypair_path),
            )?,
            verbose: matches.is_present("verbose"),
        }
    };
    solana_logger::setup_with_default("solana=info");
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    match (sub_command, sub_matches) {
        ("show-house", Some(arg_matches)) => {
            let user_address =
                pubkey_of(arg_matches, "user_address").unwrap_or(config.keypair.pubkey());
            let house_addr = nobilitydao::get_house_address(&user_address);
            println!("House Address: {}", house_addr);
            let housedata = get_house(&rpc_client, &house_addr)?;
            let coa_url = std::str::from_utf8(&housedata.coat_of_arms)?;
            let display_name = std::str::from_utf8(&housedata.display_name)?;
            println!("Display Name: {}", display_name);
            println!("Coat of Arms: {}", coa_url);
            Ok(())
        }
        ("create-house", Some(arg_matches)) => {
            let user_keypair = keypair_of(arg_matches, "user_address").unwrap_or(config.keypair);
            let coat_of_arms_str = arg_matches.value_of("coat_of_arms").unwrap();
            let display_name_str = arg_matches.value_of("display_name").unwrap();
            create_house(
                &rpc_client,
                &user_keypair,
                coat_of_arms_str,
                display_name_str,
            )
        }
        ("show-root-title", Some(arg_matches)) => {
            let liege_address = Pubkey::new(&[0; 32]);
            let title_address = nobilitydao::get_title_address(&liege_address, 0);
            println!("Title address: {}", title_address);
            let titledata = get_title(&rpc_client, &title_address)?;
            print_title(&titledata)
        }
        ("show-title", Some(arg_matches)) => {
            let title_address = pubkey_of(arg_matches, "title_address").unwrap();
            println!("Title address: {}", title_address);
            let titledata = get_title(&rpc_client, &title_address)?;
            print_title(&titledata)
        }
        ("create-title", Some(arg_matches)) => {
            let user_keypair = keypair_of(arg_matches, "user_address").unwrap_or(config.keypair);
            let liege_title_address = if arg_matches.value_of("liege_address").unwrap().len() == 0 {
                Pubkey::new(&[0; 32])
            } else {
                pubkey_of(arg_matches, "liege_address").unwrap()
            };
            let rank = value_t_or_exit!(arg_matches, "rank", u8);
            let kind = value_t_or_exit!(arg_matches, "kind", u8);
            let required_stake_lamports =
                value_t_or_exit!(arg_matches, "required_stake_lamports", u64);
            let liege_vassal_index = value_t_or_exit!(arg_matches, "liege_vassal_index", u8);
            let coat_of_arms_str = arg_matches.value_of("coat_of_arms").unwrap();
            let display_name_str = arg_matches.value_of("display_name").unwrap();
            create_title(
                &rpc_client,
                &user_keypair,
                &liege_title_address,
                rank,
                kind,
                required_stake_lamports,
                liege_vassal_index,
                coat_of_arms_str,
                display_name_str,
            )
        }
        _ => unreachable!(),
    }
}

fn get_house(rpc_client: &RpcClient, house_address: &Pubkey) -> Result<HouseData, String> {
    let account = rpc_client
        .get_multiple_accounts(&[*house_address])
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .unwrap();

    match account {
        None => Err(format!("House {} does not exist", house_address)),
        Some(account) => HouseData::try_from_slice(&account.data)
            .map_err(|err| format!("Failed to deserialize house {}: {}", house_address, err)),
    }
}

fn get_title(rpc_client: &RpcClient, title_address: &Pubkey) -> Result<TitleData, String> {
    let account = rpc_client
        .get_multiple_accounts(&[*title_address])
        .map_err(|err| err.to_string())?
        .into_iter()
        .next()
        .unwrap();

    match account {
        None => Err(format!("Title {} does not exist", title_address)),
        Some(account) => {
            let liege_title_data: Result<TitleData, std::io::Error> = {
                let v = &account.data.as_slice();
                let mut v_mut: &[u8] = *v;
                let r = TitleData::deserialize(&mut v_mut);
                r
            };
            liege_title_data.map_err(|err| err.to_string())
            // TitleData::try_from_slice(&account.data)
            // .map_err(|err| format!("Failed to deserialize title {}: {}", title_address, err))
        },
    }
}

fn print_title(titledata: &TitleData) -> Result<(), Box<dyn std::error::Error>>  {
    let coa_url = std::str::from_utf8(&titledata.coat_of_arms)?;
    let display_name = std::str::from_utf8(&titledata.display_name)?;
    println!("Display Name: {}", display_name);
    println!("Coat of Arms: {}", coa_url);
    println!("Rank: {}", titledata.rank);
    println!("Kind: {}", titledata.kind);
    println!("Required stake (SOL): {}", lamports_to_sol(titledata.required_stake_lamports));
    println!("Sale price (SOL): {}", lamports_to_sol(titledata.sale_price_lamports));
    println!("Holder: {}", titledata.holder_house_address);
    if titledata.liege_address != Pubkey::new(&[0;32]) {
        println!("Liege: {}", titledata.liege_address);
    }
    for vassal_address in titledata.vassal_addresses.iter() {
        println!("Vassal: {}", vassal_address);
    }
    Ok(())
}

// fn unix_timestamp_to_string(unix_timestamp: UnixTimestamp) -> String {
//     format!(
//         "{} (UnixTimestamp: {})",
//         match NaiveDateTime::from_timestamp_opt(unix_timestamp, 0) {
//             Some(ndt) =>
//                 DateTime::<Utc>::from_utc(ndt, Utc).to_rfc3339_opts(SecondsFormat::Secs, true),
//             None => "unknown".to_string(),
//         },
//         unix_timestamp,
//     )
// }

fn create_house(
    rpc_client: &RpcClient,
    user_keypair: &Keypair,
    coat_of_arms_str: &str,
    display_name_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let house_addr = nobilitydao::get_house_address(&user_keypair.pubkey());
    println!("House Address: {}", house_addr);
    let coat_of_arms: &mut [u8; 128] = &mut [0; 128];
    let coa_str_bytes = coat_of_arms_str.as_bytes();
    coat_of_arms[..coa_str_bytes.len()].clone_from_slice(&coa_str_bytes);

    let display_name: &mut [u8; 128] = &mut [0; 128];
    let display_name_str_bytes = display_name_str.as_bytes();
    display_name[..display_name_str.len()].clone_from_slice(&display_name_str_bytes);

    let mut transaction = Transaction::new_with_payer(
        &[nobilitydao::instruction::create_house(
            &user_keypair.pubkey(),
            &house_addr,
            &coat_of_arms,
            &display_name,
        )],
        Some(&user_keypair.pubkey()),
    );
    let blockhash = rpc_client.get_recent_blockhash()?.0;
    transaction.try_sign(&[user_keypair], blockhash)?;

    rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
    println!("Done creating house!");
    Ok(())
}

fn create_title(
    rpc_client: &RpcClient,
    user_keypair: &Keypair,
    liege_address: &Pubkey,
    rank: u8,
    kind: u8,
    required_stake_lamports: u64,
    liege_vassal_index: u8,
    coat_of_arms_str: &str,
    display_name_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let house_addr = nobilitydao::get_house_address(&user_keypair.pubkey());
    let new_title_addr = nobilitydao::get_title_address(liege_address, liege_vassal_index);
    println!("House Address: {}", house_addr);
    println!("New title Address: {}", new_title_addr);

    let coat_of_arms: &mut [u8; 128] = &mut [0; 128];
    let coa_str_bytes = coat_of_arms_str.as_bytes();
    coat_of_arms[..coa_str_bytes.len()].clone_from_slice(&coa_str_bytes);

    let display_name: &mut [u8; 128] = &mut [0; 128];
    let display_name_str_bytes = display_name_str.as_bytes();
    display_name[..display_name_str.len()].clone_from_slice(&display_name_str_bytes);

    let mut transaction = Transaction::new_with_payer(
        &[nobilitydao::instruction::create_title(
            &user_keypair.pubkey(),
            &house_addr,
            &new_title_addr,
            liege_address,
            rank,
            kind,
            required_stake_lamports,
            liege_vassal_index,
            &coat_of_arms,
            &display_name,
        )],
        Some(&user_keypair.pubkey()),
    );
    let blockhash = rpc_client.get_recent_blockhash()?.0;
    transaction.try_sign(&[user_keypair], blockhash)?;

    rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
    println!("Done creating title!");
    Ok(())
}

// fn process_propose(
//     rpc_client: &RpcClient,
//     config: &Config,
//     feature_proposal_keypair: &Keypair,
//     distribution_file: String,
//     percent_stake_required: u8,
//     deadline: UnixTimestamp,
//     confirm: bool,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let distributor_token_address =
//         spl_feature_proposal::get_distributor_token_address(&feature_proposal_keypair.pubkey());
//     let feature_id_address =
//         spl_feature_proposal::get_feature_id_address(&feature_proposal_keypair.pubkey());
//     let acceptance_token_address =
//         spl_feature_proposal::get_acceptance_token_address(&feature_proposal_keypair.pubkey());
//     let mint_address = spl_feature_proposal::get_mint_address(&feature_proposal_keypair.pubkey());

//     println!("Feature Id: {}", feature_id_address);
//     println!("Token Mint Address: {}", mint_address);
//     println!("Distributor Token Address: {}", distributor_token_address);
//     println!("Acceptance Token Address: {}", acceptance_token_address);

//     let vote_accounts = rpc_client.get_vote_accounts()?;
//     let mut distribution = HashMap::new();
//     for (pubkey, activated_stake) in vote_accounts
//         .current
//         .into_iter()
//         .chain(vote_accounts.delinquent)
//         .map(|vote_account| (vote_account.node_pubkey, vote_account.activated_stake))
//     {
//         distribution
//             .entry(pubkey)
//             .and_modify(|e| *e += activated_stake)
//             .or_insert(activated_stake);
//     }

//     let tokens_to_mint: u64 = distribution.iter().map(|x| x.1).sum();
//     let tokens_required = tokens_to_mint * percent_stake_required as u64 / 100;

//     println!("Number of validators: {}", distribution.len());
//     println!(
//         "Tokens to be minted: {}",
//         spl_feature_proposal::amount_to_ui_amount(tokens_to_mint)
//     );
//     println!(
//         "Tokens required for acceptance: {} ({}%)",
//         spl_feature_proposal::amount_to_ui_amount(tokens_required),
//         percent_stake_required
//     );

//     println!("Token distribution file: {}", distribution_file);
//     {
//         let mut file = File::create(&distribution_file)?;
//         file.write_all(b"recipient,amount\n")?;
//         for (node_address, activated_stake) in distribution.iter() {
//             file.write_all(format!("{},{}\n", node_address, activated_stake).as_bytes())?;
//         }
//     }

//     let mut transaction = Transaction::new_with_payer(
//         &[spl_feature_proposal::instruction::propose(
//             &config.keypair.pubkey(),
//             &feature_proposal_keypair.pubkey(),
//             tokens_to_mint,
//             AcceptanceCriteria {
//                 tokens_required,
//                 deadline,
//             },
//         )],
//         Some(&config.keypair.pubkey()),
//     );
//     let blockhash = rpc_client.get_recent_blockhash()?.0;
//     transaction.try_sign(&[&config.keypair, feature_proposal_keypair], blockhash)?;

//     println!("JSON RPC URL: {}", config.json_rpc_url);

//     println!();
//     println!("Distribute the proposal tokens to all validators by running:");
//     println!(
//         "    $ solana-tokens distribute-spl-tokens \
//                   --from {} \
//                   --input-csv {} \
//                   --db-path db.{} \
//                   --fee-payer ~/.config/solana/id.json \
//                   --owner <FEATURE_PROPOSAL_KEYPAIR>",
//         distributor_token_address,
//         distribution_file,
//         &feature_proposal_keypair.pubkey().to_string()[..8]
//     );
//     println!(
//         "    $ solana-tokens spl-token-balances \
//                  --mint {} --input-csv {}",
//         mint_address, distribution_file
//     );
//     println!();

//     println!(
//         "Once the distribution is complete, request validators vote for \
//         the proposal by first looking up their token account address:"
//     );
//     println!(
//         "    $ spl-token --owner ~/validator-keypair.json accounts {}",
//         mint_address
//     );
//     println!("and then submit their vote by running:");
//     println!(
//         "    $ spl-token --owner ~/validator-keypair.json transfer <TOKEN_ACCOUNT_ADDRESS> ALL {}",
//         acceptance_token_address
//     );
//     println!();
//     println!("Periodically the votes must be tallied by running:");
//     println!(
//         "  $ spl-feature-proposal tally {}",
//         feature_proposal_keypair.pubkey()
//     );
//     println!("Tallying is permissionless and may be run by anybody.");
//     println!("Once this feature proposal is accepted, the {} feature will be activated at the next epoch.", feature_id_address);

//     println!();
//     println!(
//         "Proposal will expire at {}",
//         unix_timestamp_to_string(deadline)
//     );
//     println!();
//     if !confirm {
//         println!("Add --confirm flag to initiate the feature proposal");
//         return Ok(());
//     }
//     rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

//     println!();
//     println!("Feature proposal created!");
//     Ok(())
// }

// fn process_tally(
//     rpc_client: &RpcClient,
//     config: &Config,
//     feature_proposal_address: &Pubkey,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let feature_proposal = get_feature_proposal(rpc_client, feature_proposal_address)?;

//     let feature_id_address = spl_feature_proposal::get_feature_id_address(feature_proposal_address);
//     let acceptance_token_address =
//         spl_feature_proposal::get_acceptance_token_address(feature_proposal_address);

//     println!("Feature Id: {}", feature_id_address);
//     println!("Acceptance Token Address: {}", acceptance_token_address);

//     match feature_proposal {
//         FeatureProposal::Uninitialized => {
//             return Err("Feature proposal is uninitialized".into());
//         }
//         FeatureProposal::Pending(acceptance_criteria) => {
//             let acceptance_token_address =
//                 spl_feature_proposal::get_acceptance_token_address(feature_proposal_address);
//             let acceptance_token_balance = rpc_client
//                 .get_token_account_balance(&acceptance_token_address)?
//                 .amount
//                 .parse::<u64>()
//                 .unwrap_or(0);

//             println!();
//             println!(
//                 "{} tokens required to accept the proposal",
//                 spl_feature_proposal::amount_to_ui_amount(acceptance_criteria.tokens_required)
//             );
//             println!(
//                 "{} tokens have been received",
//                 spl_feature_proposal::amount_to_ui_amount(acceptance_token_balance)
//             );
//             println!(
//                 "Proposal will expire at {}",
//                 unix_timestamp_to_string(acceptance_criteria.deadline)
//             );
//             println!();

//             // Don't bother issuing a transaction if it's clear the Tally won't succeed
//             if acceptance_token_balance < acceptance_criteria.tokens_required
//                 && (SystemTime::now()
//                     .duration_since(UNIX_EPOCH)
//                     .unwrap()
//                     .as_secs() as UnixTimestamp)
//                     < acceptance_criteria.deadline
//             {
//                 println!("Feature proposal pending");
//                 return Ok(());
//             }
//         }
//         FeatureProposal::Accepted { .. } => {
//             println!("Feature proposal accepted");
//             return Ok(());
//         }
//         FeatureProposal::Expired => {
//             println!("Feature proposal expired");
//             return Ok(());
//         }
//     }

//     let mut transaction = Transaction::new_with_payer(
//         &[spl_feature_proposal::instruction::tally(
//             feature_proposal_address,
//         )],
//         Some(&config.keypair.pubkey()),
//     );
//     let blockhash = rpc_client.get_recent_blockhash()?.0;
//     transaction.try_sign(&[&config.keypair], blockhash)?;

//     rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;

//     // Check the status of the proposal after the tally completes
//     let feature_proposal = get_feature_proposal(rpc_client, feature_proposal_address)?;
//     match feature_proposal {
//         FeatureProposal::Uninitialized => Err("Feature proposal is uninitialized".into()),
//         FeatureProposal::Pending { .. } => {
//             println!("Feature proposal pending");
//             Ok(())
//         }
//         FeatureProposal::Accepted { .. } => {
//             println!("Feature proposal accepted");
//             Ok(())
//         }
//         FeatureProposal::Expired => {
//             println!("Feature proposal expired");
//             Ok(())
//         }
//     }
// }
