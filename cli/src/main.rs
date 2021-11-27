use {
    clap::{
        crate_description, crate_name, crate_version, value_t_or_exit, App, AppSettings, Arg,
        SubCommand,
    },
    nobilitydao::{
        state::{HouseData, TitleData, MAX_KIND, MAX_RANK, MAX_VASSALS, MIN_KIND, MIN_RANK},
        utils::try_from_slice_checked,
    },
    solana_clap_utils::{
        input_parsers::{keypair_of, pubkey_of},
        input_validators::{
            is_keypair, is_url, is_valid_pubkey, is_within_range,
        },
    },
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::lamports_to_sol,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    },
    std::{
        fmt::Display,
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
            let coa_url = housedata.coat_of_arms;
            let display_name = housedata.display_name;
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
        Some(account) => try_from_slice_checked::<HouseData>(&account.data, HouseData::SIZE)
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
        Some(account) => try_from_slice_checked::<TitleData>(&account.data, TitleData::SIZE)
            .map_err(|err| format!("Failed to deserialize title {}: {}", title_address, err)),
    }
}

fn print_title(titledata: &TitleData) -> Result<(), Box<dyn std::error::Error>> {
    let coa_url = &titledata.coat_of_arms;
    let display_name = &titledata.display_name;
    println!("Display Name: {}", display_name);
    println!("Coat of Arms: {}", coa_url);
    println!("Rank: {}", titledata.rank);
    println!("Kind: {}", titledata.kind);
    println!(
        "Required stake (SOL): {}",
        lamports_to_sol(titledata.required_stake_lamports)
    );
    println!(
        "Sale price (SOL): {}",
        lamports_to_sol(titledata.sale_price_lamports)
    );
    println!("Holder: {}", titledata.holder_house_address);
    if titledata.liege_address != Pubkey::new(&[0; 32]) {
        println!("Liege: {}", titledata.liege_address);
    }
    for vassal_address in titledata.vassal_addresses.iter() {
        println!("Vassal: {}", vassal_address);
    }
    Ok(())
}

fn create_house(
    rpc_client: &RpcClient,
    user_keypair: &Keypair,
    coat_of_arms_str: &str,
    display_name_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let house_addr = nobilitydao::get_house_address(&user_keypair.pubkey());
    println!("House Address: {}", house_addr);

    let mut transaction = Transaction::new_with_payer(
        &[nobilitydao::instruction::create_house(
            &user_keypair.pubkey(),
            &house_addr,
            coat_of_arms_str.to_string(),
            display_name_str.to_string(),
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
            coat_of_arms_str.to_string(),
            display_name_str.to_string(),
        )],
        Some(&user_keypair.pubkey()),
    );
    let blockhash = rpc_client.get_recent_blockhash()?.0;
    transaction.try_sign(&[user_keypair], blockhash)?;

    rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
    println!("Done creating title!");
    Ok(())
}
